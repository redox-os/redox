use alloc::boxed::Box;

use collections::string::ToString;
use collections::vec::Vec;

use core::mem;

use network::common::*;
use network::ipv4::*;

use common::{debug, random};
use common::to_num::ToNum;
use common::parse_ip::*;

use schemes::arp::{Arp, ArpHeader};
use schemes::{KScheme, Resource, Url};

/// A IP (internet protocole) resource
pub struct IpResource {
    link: Box<Resource>,
    data: Vec<u8>,
    peer_addr: Ipv4Addr,
    proto: u8,
    id: u16,
}

impl Resource for IpResource {
    fn dup(&self) -> Option<Box<Resource>> {
        match self.link.dup() {
            Some(link) => Some(box IpResource {
                link: link,
                data: self.data.clone(),
                peer_addr: self.peer_addr,
                proto: self.proto,
                id: self.id,
            }),
            None => None
        }
    }

    fn url(&self) -> Url {
        Url::from_string(format!("ip://{}/{:X}", self.peer_addr.to_string(), self.proto))
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        debug::d("TODO: Implement read for ip://\n");
        None
    }

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        if !self.data.is_empty() {
            let mut bytes: Vec<u8> = Vec::new();
            mem::swap(&mut self.data, &mut bytes);
            vec.push_all(&bytes);
            return Some(bytes.len());
        }

        loop {
            let mut bytes: Vec<u8> = Vec::new();
            match self.link.read_to_end(&mut bytes) {
                Some(_) => {
                    if let Some(packet) = Ipv4::from_bytes(bytes) {
                        if packet.header.proto == self.proto && packet.header.dst.equals(IP_ADDR) &&
                           packet.header.src.equals(self.peer_addr) {
                            vec.push_all(&packet.data);
                            return Some(packet.data.len());
                        }
                    }
                }
                None => return None,
            }
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        let ip_data = Vec::from(buf);

        self.id += 1;
        let mut ip = Ipv4 {
            header: Ipv4Header {
                ver_hlen: 0x40 | (mem::size_of::<Ipv4Header>()/4 & 0xF) as u8, // No Options
                services: 0,
                len: n16::new((mem::size_of::<Ipv4Header>() + ip_data.len()) as u16), // No Options
                id: n16::new(self.id),
                flags_fragment: n16::new(0),
                ttl: 128,
                proto: self.proto,
                checksum: Checksum { data: 0 },
                src: IP_ADDR,
                dst: self.peer_addr,
            },
            options: Vec::new(),
            data: ip_data,
        };

        unsafe {
            let header_ptr: *const Ipv4Header = &ip.header;
            ip.header.checksum.data =
                Checksum::compile(Checksum::sum(header_ptr as usize, mem::size_of::<Ipv4Header>()) +
                                  Checksum::sum(ip.options.as_ptr() as usize, ip.options.len()));
        }

        match self.link.write(&ip.to_bytes()) {
            Some(_) => Some(buf.len()),
            None => None,
        }
    }

    fn sync(&mut self) -> bool {
        self.link.sync()
    }
}

/// A ARP entry (MAC + IP)
pub struct ArpEntry {
    ip: Ipv4Addr,
    mac: MacAddr,
}

/// A IP scheme
pub struct IpScheme {
    pub arp: Vec<ArpEntry>,
}

impl KScheme for IpScheme {
    fn scheme(&self) -> &str {
        "ip"
    }

    fn open(&mut self, url: &Url, _: usize) -> Option<Box<Resource>> {
        if !url.reference().is_empty() {
            let proto = url.reference().to_num_radix(16) as u8;

            if !parse_host(url.reference()).is_empty() {
                let peer_addr = Ipv4Addr::from_string(&parse_host(url.reference()).to_string());
                let mut peer_mac = BROADCAST_MAC_ADDR;

                for entry in self.arp.iter() {
                    if entry.ip.equals(peer_addr) {
                        peer_mac = entry.mac;
                        break;
                    }
                }

                if peer_mac.equals(BROADCAST_MAC_ADDR) {
                    if let Some(mut link) = Url::from_string("ethernet://".to_string() + &peer_mac.to_string() + "/806").open() {
                        let arp = Arp {
                            header: ArpHeader {
                                htype: n16::new(1),
                                ptype: n16::new(0x800),
                                hlen: 6,
                                plen: 4,
                                oper: n16::new(1),
                                src_mac: unsafe { MAC_ADDR },
                                src_ip: IP_ADDR,
                                dst_mac: peer_mac,
                                dst_ip: peer_addr,
                            },
                            data: Vec::new(),
                        };

                        match link.write(&arp.to_bytes()) {
                            Some(_) => loop {
                                let mut bytes: Vec<u8> = Vec::new();
                                match link.read_to_end(&mut bytes) {
                                    Some(_) =>
                                        if let Some(packet) = Arp::from_bytes(bytes) {
                                        if packet.header.oper.get() == 2 &&
                                           packet.header.src_ip.equals(peer_addr) {
                                            peer_mac = packet.header.src_mac;
                                            self.arp.push(ArpEntry {
                                                ip: peer_addr,
                                                mac: peer_mac,
                                            });
                                            break;
                                        }
                                    },
                                    None => (),
                                }
                            },
                            None => debug::d("IP: ARP Write Failed!\n"),
                        }
                    }
                }

                if let Some(link) = Url::from_string("ethernet://".to_string() + &peer_mac.to_string() + "/800").open() {
                    return Some(box IpResource {
                        link: link,
                        data: Vec::new(),
                        peer_addr: peer_addr,
                        proto: proto,
                        id: (random::rand() % 65536) as u16,
                    });
                }
            } else {
                while let Some(mut link) = Url::from_str("ethernet:///800").open() {
                    let mut bytes: Vec<u8> = Vec::new();
                    match link.read_to_end(&mut bytes) {
                        Some(_) => {
                            if let Some(packet) = Ipv4::from_bytes(bytes) {
                                if packet.header.proto == proto &&
                                   packet.header.dst.equals(IP_ADDR) {
                                    return Some(box IpResource {
                                        link: link,
                                        data: packet.data,
                                        peer_addr: packet.header.src,
                                        proto: proto,
                                        id: (random::rand() % 65536) as u16,
                                    });
                                }
                            }
                        }
                        None => break,
                    }
                }
            }
        } else {
            debug::d("IP: No protocol provided\n");
        }

        None
    }
}
