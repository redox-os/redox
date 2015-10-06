use alloc::boxed::Box;

use core::mem;

use network::arp::*;
use network::common::*;
use network::ipv4::*;

use common::{debug, random};
use common::resource::{NoneResource, Resource, ResourceSeek, ResourceType, URL};
use common::string::{String, ToString};
use common::vec::Vec;

use programs::common::SessionItem;

pub struct IPResource {
    link: Box<Resource>,
    data: Vec<u8>,
    peer_addr: IPv4Addr,
    proto: u8,
    id: u16,
}

impl Resource for IPResource {
    fn url(&self) -> URL {
        return URL::from_string(&("ip://".to_string() + self.peer_addr.to_string() + '/' +
                                  String::from_num_radix(self.proto as usize, 16)));
    }

    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        debug::d("TODO: Implement read for ip://\n");
        return Option::None;
    }

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        if self.data.len() > 0 {
            let mut bytes: Vec<u8> = Vec::new();
            mem::swap(&mut self.data, &mut bytes);
            vec.push_all(&bytes);
            return Option::Some(bytes.len());
        }

        loop {
            let mut bytes: Vec<u8> = Vec::new();
            match self.link.read_to_end(&mut bytes) {
                Option::Some(_) => {
                    if let Option::Some(packet) = IPv4::from_bytes(bytes) {
                        if packet.header.proto == self.proto && packet.header.dst.equals(IP_ADDR) &&
                           packet.header.src.equals(self.peer_addr) {
                            vec.push_all(&packet.data);
                            return Option::Some(packet.data.len());
                        }
                    }
                }
                Option::None => return Option::None,
            }
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        let ip_data = unsafe { Vec::from_raw_buf(buf.as_ptr(), buf.len()) };

        self.id += 1;
        let mut ip = IPv4 {
            header: IPv4Header {
                ver_hlen: 0x40 | (mem::size_of::<IPv4Header>()/4 & 0xF) as u8, // No Options
                services: 0,
                len: n16::new((mem::size_of::<IPv4Header>() + ip_data.len()) as u16), // No Options
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
            let header_ptr: *const IPv4Header = &ip.header;
            ip.header.checksum.data =
                Checksum::compile(Checksum::sum(header_ptr as usize, mem::size_of::<IPv4Header>()) +
                                  Checksum::sum(ip.options.as_ptr() as usize, ip.options.len()));
        }

        match self.link.write(ip.to_bytes().as_slice()) {
            Option::Some(_) => return Option::Some(buf.len()),
            Option::None => return Option::None,
        }
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None;
    }

    fn sync(&mut self) -> bool {
        return self.link.sync();
    }
}

pub struct ARPEntry {
    ip: IPv4Addr,
    mac: MACAddr,
}

pub struct IPScheme {
    pub arp: Vec<ARPEntry>,
}

impl SessionItem for IPScheme {
    fn scheme(&self) -> String {
        return "ip".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        if url.path().len() > 0 {
            let proto = url.path().to_num_radix(16) as u8;

            if url.host().len() > 0 {
                let peer_addr = IPv4Addr::from_string(&url.host());
                let mut peer_mac = BROADCAST_MAC_ADDR;

                for entry in self.arp.iter() {
                    if entry.ip.equals(peer_addr) {
                        peer_mac = entry.mac;
                        break;
                    }
                }

                if peer_mac.equals(BROADCAST_MAC_ADDR) {
                    let mut link = URL::from_string(&("ethernet://".to_string() +
                                                      peer_mac.to_string() +
                                                      "/806"))
                                       .open();

                    let arp = ARP {
                        header: ARPHeader {
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

                    match link.write(arp.to_bytes().as_slice()) {
                        Option::Some(_) => loop {
                            let mut bytes: Vec<u8> = Vec::new();
                            match link.read_to_end(&mut bytes) {
                                Option::Some(_) =>
                                    if let Option::Some(packet) = ARP::from_bytes(bytes) {
                                    if packet.header.oper.get() == 2 &&
                                       packet.header.src_ip.equals(peer_addr) {
                                        peer_mac = packet.header.src_mac;
                                        self.arp.push(ARPEntry {
                                            ip: peer_addr,
                                            mac: peer_mac,
                                        });
                                        break;
                                    }
                                },
                                Option::None => (),
                            }
                        },
                        Option::None => debug::d("IP: ARP Write Failed!\n")
                    }
                }

                return box IPResource {
                    link: URL::from_string(&("ethernet://".to_string() + peer_mac.to_string() +
                                             "/800"))
                              .open(),
                    data: Vec::new(),
                    peer_addr: peer_addr,
                    proto: proto,
                    id: (random::rand() % 65536) as u16,
                };
            } else {
                loop {
                    let mut link = URL::from_str("ethernet:///800").open();

                    let mut bytes: Vec<u8> = Vec::new();
                    match link.read_to_end(&mut bytes) {
                        Option::Some(_) => {
                            if let Option::Some(packet) = IPv4::from_bytes(bytes) {
                                if packet.header.proto == proto &&
                                   packet.header.dst.equals(IP_ADDR) {
                                    return box IPResource {
                                        link: link,
                                        data: packet.data,
                                        peer_addr: packet.header.src,
                                        proto: proto,
                                        id: (random::rand() % 65536) as u16,
                                    };
                                }
                            }
                        }
                        Option::None => break,
                    }
                }
            }
        } else {
            debug::d("IP: No protocol provided\n");
        }

        return box NoneResource;
    }
}
