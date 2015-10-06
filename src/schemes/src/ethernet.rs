use alloc::boxed::Box;

use core::mem::swap;

use network::common::*;
use network::ethernet::*;

use common::debug;
use common::resource::{NoneResource, Resource, ResourceSeek, ResourceType, URL};
use common::string::{String, ToString};
use common::vec::Vec;

use programs::common::SessionItem;

pub struct EthernetResource {
    network: Box<Resource>,
    data: Vec<u8>,
    peer_addr: MACAddr,
    ethertype: u16,
}

impl Resource for EthernetResource {
    fn url(&self) -> URL {
        return URL::from_string(&("ethernet://".to_string() + self.peer_addr.to_string() + '/' +
                                  String::from_num_radix(self.ethertype as usize, 16)));
    }

    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        debug::d("TODO: Implement read for ethernet://\n");
        return Option::None;
    }

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        if self.data.len() > 0 {
            let mut bytes: Vec<u8> = Vec::new();
            swap(&mut self.data, &mut bytes);
            vec.push_all(&bytes);
            return Option::Some(bytes.len());
        }

        loop {
            let mut bytes: Vec<u8> = Vec::new();
            match self.network.read_to_end(&mut bytes) {
                Option::Some(_) => {
                    if let Option::Some(frame) = EthernetII::from_bytes(bytes) {
                        if frame.header.ethertype.get() == self.ethertype &&
                           (unsafe { frame.header.dst.equals(MAC_ADDR) } ||
                            frame.header.dst.equals(BROADCAST_MAC_ADDR)) &&
                           (frame.header.src.equals(self.peer_addr) ||
                            self.peer_addr.equals(BROADCAST_MAC_ADDR)) {
                            vec.push_all(&frame.data);
                            return Option::Some(frame.data.len());
                        }
                    }
                }
                Option::None => return Option::None,
            }
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        let data = unsafe { Vec::from_raw_buf(buf.as_ptr(), buf.len()) };

        match self.network.write(EthernetII {
            header: EthernetIIHeader {
                src: unsafe { MAC_ADDR },
                dst: self.peer_addr,
                ethertype: n16::new(self.ethertype),
            },
            data: data,
        }
                                     .to_bytes()
                                     .as_slice()) {
            Option::Some(_) => return Option::Some(buf.len()),
            Option::None => return Option::None,
        }
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None;
    }

    fn sync(&mut self) -> bool {
        return self.network.sync();
    }
}

pub struct EthernetScheme;

impl SessionItem for EthernetScheme {
    fn scheme(&self) -> String {
        return "ethernet".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        let mut network = URL::from_str("network://").open();

        if url.path().len() > 0 {
            let ethertype = url.path().to_num_radix(16) as u16;

            if url.host().len() > 0 {
                return box EthernetResource {
                    network: network,
                    data: Vec::new(),
                    peer_addr: MACAddr::from_string(&url.host()),
                    ethertype: ethertype,
                };
            } else {
                loop {
                    let mut bytes: Vec<u8> = Vec::new();
                    match network.read_to_end(&mut bytes) {
                        Option::Some(_) => {
                            if let Option::Some(frame) = EthernetII::from_bytes(bytes) {
                                if frame.header.ethertype.get() == ethertype &&
                                   (unsafe { frame.header.dst.equals(MAC_ADDR) } ||
                                    frame.header.dst.equals(BROADCAST_MAC_ADDR)) {
                                    return box EthernetResource {
                                        network: network,
                                        data: frame.data,
                                        peer_addr: frame.header.src,
                                        ethertype: ethertype,
                                    };
                                }
                            }
                        }
                        Option::None => break,
                    }
                }
            }
        } else {
            debug::d("Ethernet: No ethertype provided\n");
        }

        return box NoneResource;
    }
}
