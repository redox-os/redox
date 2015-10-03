use alloc::boxed::Box;

use core::mem;

use common::{debug, random};
use common::resource::{NoneResource, Resource, ResourceSeek, ResourceType, URL};
use common::string::{String, ToString};
use common::vec::Vec;

use network::common::*;
use network::udp::*;

use programs::common::SessionItem;

pub struct UDPResource {
    ip: Box<Resource>,
    data: Vec<u8>,
    peer_addr: IPv4Addr,
    peer_port: u16,
    host_port: u16,
}

impl Resource for UDPResource {
    fn url(&self) -> URL {
        return URL::from_string(&("udp://".to_string() + self.peer_addr.to_string() + ':' +
                                  self.peer_port as usize +
                                  '/' + self.host_port as usize));
    }

    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        debug::d("TODO: Implement read for udp://\n");
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
            match self.ip.read_to_end(&mut bytes) {
                Option::Some(_) => {
                    if let Option::Some(datagram) = UDP::from_bytes(bytes) {
                        if datagram.header.dst.get() == self.host_port &&
                           datagram.header.src.get() == self.peer_port {
                            vec.push_all(&datagram.data);
                            return Option::Some(datagram.data.len());
                        }
                    }
                }
                Option::None => return Option::None,
            }
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        let udp_data = unsafe { Vec::from_raw_buf(buf.as_ptr(), buf.len()) };

        let mut udp = UDP {
            header: UDPHeader {
                src: n16::new(self.host_port),
                dst: n16::new(self.peer_port),
                len: n16::new((mem::size_of::<UDPHeader>() + udp_data.len()) as u16),
                checksum: Checksum { data: 0 },
            },
            data: udp_data,
        };

        unsafe {
            let proto = n16::new(0x11);
            let datagram_len = n16::new((mem::size_of::<UDPHeader>() + udp.data.len()) as u16);
            udp.header.checksum.data =
                Checksum::compile(Checksum::sum((&IP_ADDR as *const IPv4Addr) as usize,
                                                mem::size_of::<IPv4Addr>()) +
                                  Checksum::sum((&self.peer_addr as *const IPv4Addr) as usize,
                                                mem::size_of::<IPv4Addr>()) +
                                  Checksum::sum((&proto as *const n16) as usize, mem::size_of::<n16>()) +
                                  Checksum::sum((&datagram_len as *const n16) as usize,
                                                mem::size_of::<n16>()) +
                                  Checksum::sum((&udp.header as *const UDPHeader) as usize,
                                                mem::size_of::<UDPHeader>()) +
                                  Checksum::sum(udp.data.as_ptr() as usize, udp.data.len()));
        }

        match self.ip.write(udp.to_bytes().as_slice()) {
            Option::Some(_) => return Option::Some(buf.len()),
            Option::None => return Option::None,
        }
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None;
    }

    fn sync(&mut self) -> bool {
        return self.ip.sync();
    }
}

pub struct UDPScheme;

impl SessionItem for UDPScheme {
    fn scheme(&self) -> String {
        return "udp".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        if url.host().len() > 0 && url.port().len() > 0 {
            let peer_port = url.port().to_num() as u16;
            let peer_addr = IPv4Addr::from_string(&url.host());
            let host_port = (random::rand() % 32768 + 32768) as u16;

            return box UDPResource {
                ip: URL::from_string(&("ip://".to_string() + peer_addr.to_string() + "/11")).open(),
                data: Vec::new(),
                peer_addr: peer_addr,
                peer_port: peer_port,
                host_port: host_port,
            };
        } else if url.path().len() > 0 {
            let host_port = url.path().to_num() as u16;
            loop {
                let mut ip = URL::from_str("ip:///11").open();

                let mut bytes: Vec<u8> = Vec::new();
                match ip.read_to_end(&mut bytes) {
                    Option::Some(_) => {
                        if let Option::Some(datagram) = UDP::from_bytes(bytes) {
                            if datagram.header.dst.get() == host_port {
                                let peer_addr = IPv4Addr::from_string(&ip.url().host());

                                return box UDPResource {
                                    ip: ip,
                                    data: datagram.data,
                                    peer_addr: peer_addr,
                                    peer_port: datagram.header.src.get(),
                                    host_port: host_port,
                                };
                            }
                        }
                    }
                    Option::None => break,
                }
            }
        } else {
            debug::d("UDP: No remote endpoint or local port provided\n");
        }

        return box NoneResource;
    }
}
