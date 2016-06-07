use alloc::boxed::Box;

use collections::string::ToString;
use collections::vec::Vec;

use core::{cmp, mem};

use network::common::*;
use network::ipv4::*;

use common::{debug, random};
use common::to_num::ToNum;

use super::arp::{Arp, ArpHeader};
use fs::{KScheme, Resource, Url};

use system::error::{ENOENT, Error, Result};

/// A IP (internet protocole) resource
pub struct IpResource {
    link: Box<Resource>,
    data: Vec<u8>,
    peer_addr: Ipv4Addr,
    proto: u8,
    id: u16,
}

impl Resource for IpResource {
    fn dup(&self) -> Result<Box<Resource>> {
        match self.link.dup() {
            Ok(link) => {
                Ok(box IpResource {
                    link: link,
                    data: self.data.clone(),
                    peer_addr: self.peer_addr,
                    proto: self.proto,
                    id: self.id,
                })
            },
            Err(err) => Err(err),
        }
    }

    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let path_string = format!("ip:{}/{:X}", self.peer_addr.to_string(), self.proto);
        let path = path_string.as_bytes();

        for (b, p) in buf.iter_mut().zip(path.iter()) {
            *b = *p;
        }

        Ok(cmp::min(buf.len(), path.len()))
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if !self.data.is_empty() {
            let mut data: Vec<u8> = Vec::new();
            mem::swap(&mut self.data, &mut data);

            for (b, d) in buf.iter_mut().zip(data.iter()) {
                *b = *d;
            }

            return Ok(cmp::min(buf.len(), data.len()));
        }

        loop {
            let mut bytes = [0; 8192];
            match self.link.read(&mut bytes) {
                Ok(count) => {
                    if let Some(packet) = Ipv4::from_bytes(bytes[..count].to_vec()) {
                        if packet.header.proto == self.proto && packet.header.dst.equals(IP_ADDR) &&
                           packet.header.src.equals(self.peer_addr) {
                            for (b, d) in buf.iter_mut().zip(packet.data.iter()) {
                                *b = *d;
                            }

                            return Ok(cmp::min(buf.len(), packet.data.len()));
                        }
                    }
                },
                Err(err) => return Err(err),
            }
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let ip_data = Vec::from(buf);

        self.id += 1;
        let mut ip = Ipv4 {
            header: Ipv4Header {
                ver_hlen: 0x40 | (mem::size_of::<Ipv4Header>() / 4 & 0xF) as u8, // No Options
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
            Ok(_) => Ok(buf.len()),
            Err(err) => Err(err),
        }
    }

    fn sync(&mut self) -> Result<()> {
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

    fn open(&mut self, url: Url, _: usize) -> Result<Box<Resource>> {
        let parts: Vec<&str> = url.reference().split('/').collect();
        if let Some(host_string) = parts.get(0) {
            if let Some(proto_string) = parts.get(1) {
                let proto = proto_string.to_num_radix(16) as u8;

                if !host_string.is_empty() {
                    let peer_addr = Ipv4Addr::from_string(&host_string.to_string());
                    let mut peer_mac = BROADCAST_MAC_ADDR;

                    for entry in self.arp.iter() {
                        if entry.ip.equals(peer_addr) {
                            peer_mac = entry.mac;
                            break;
                        }
                    }

                    if peer_mac.equals(BROADCAST_MAC_ADDR) {
                        if let Ok(mut link) = Url::from_str(&format!("ethernet:{}/806",
                                                                     &peer_mac.to_string()))
                            .unwrap()
                            .open() {
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
                                Ok(_) => {
                                    loop {
                                        let mut bytes = [0; 8192];
                                        match link.read(&mut bytes) {
                                            Ok(count) => {
                                                if let Some(packet) =
                                                       Arp::from_bytes(bytes[..count].to_vec()) {
                                                    if packet.header.oper.get() == 2 &&
                                                       packet.header.src_ip.equals(peer_addr) {
                                                        peer_mac = packet.header.src_mac;
                                                        self.arp.push(ArpEntry {
                                                            ip: peer_addr,
                                                            mac: peer_mac,
                                                        });
                                                        break;
                                                    }
                                                }
                                            },
                                            Err(_) => (),
                                        }
                                    }
                                },
                                Err(err) => debugln!("IP: ARP Write Failed: {}", err),
                            }
                        }
                    }

                    if let Ok(link) = Url::from_str(&format!("ethernet:{}/800",
                                                             &peer_mac.to_string()))
                        .unwrap()
                        .open() {
                        return Ok(box IpResource {
                            link: link,
                            data: Vec::new(),
                            peer_addr: peer_addr,
                            proto: proto,
                            id: (random::rand() % 65536) as u16,
                        });
                    }
                } else {
                    while let Ok(mut link) = Url::from_str("ethernet:/800").unwrap().open() {
                        let mut bytes = [0; 8192];
                        match link.read(&mut bytes) {
                            Ok(count) => {
                                if let Some(packet) = Ipv4::from_bytes(bytes[..count].to_vec()) {
                                    if packet.header.proto == proto &&
                                       packet.header.dst.equals(IP_ADDR) {
                                        return Ok(box IpResource {
                                            link: link,
                                            data: packet.data,
                                            peer_addr: packet.header.src,
                                            proto: proto,
                                            id: (random::rand() % 65536) as u16,
                                        });
                                    }
                                }
                            },
                            Err(_) => break,
                        }
                    }
                }
            } else {
                debug::d("IP: No protocol provided\n");
            }
        } else {
            debug::d("IP: No host provided\n");
        }

        Err(Error::new(ENOENT))
    }
}
