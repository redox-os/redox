use alloc::boxed::Box;

use core::mem;

use network::common::*;
use network::ethernet::*;

use common::debug;
use common::string::{String, ToString};
use common::vec::Vec;

use schemes::{KScheme, Resource, ResourceSeek, URL};

/// A ethernet resource
pub struct EthernetResource {
    /// The network
    network: Box<Resource>,
    /// The data
    data: Vec<u8>,
    /// The MAC addresss
    peer_addr: MACAddr,
    /// The ethernet type
    ethertype: u16,
}

impl Resource for EthernetResource {
    fn dup(&self) -> Option<Box<Resource>> {
        match self.network.dup() {
            Some(network) => Some(box EthernetResource {
                network: network,
                data: self.data.clone(),
                peer_addr: self.peer_addr,
                ethertype: self.ethertype,
            }),
            None => None
        }
    }

    fn url(&self) -> URL {
        URL::from_string(&("ethernet://".to_string() + self.peer_addr.to_string() + '/' +
                                  String::from_num_radix(self.ethertype as usize, 16)))
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        debug::d("TODO: Implement read for ethernet://\n");
        return None;
    }

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        if self.data.len() > 0 {
            let mut bytes: Vec<u8> = Vec::new();
            mem::swap(&mut self.data, &mut bytes);
            vec.push_all(&bytes);
            return Some(bytes.len());
        }

        loop {
            let mut bytes: Vec<u8> = Vec::new();
            match self.network.read_to_end(&mut bytes) {
                Some(_) => {
                    if let Some(frame) = EthernetII::from_bytes(bytes) {
                        if frame.header.ethertype.get() == self.ethertype &&
                           (unsafe { frame.header.dst.equals(MAC_ADDR) } ||
                            frame.header.dst.equals(BROADCAST_MAC_ADDR)) &&
                           (frame.header.src.equals(self.peer_addr) ||
                            self.peer_addr.equals(BROADCAST_MAC_ADDR)) {
                            vec.push_all(&frame.data);
                            return Some(frame.data.len());
                        }
                    }
                }
                None => return None,
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
            Some(_) => return Some(buf.len()),
            None => return None,
        }
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return None;
    }

    fn sync(&mut self) -> bool {
        return self.network.sync();
    }
}

pub struct EthernetScheme;

impl KScheme for EthernetScheme {
    fn scheme(&self) -> String {
        return "ethernet".to_string();
    }

    fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
        if let Some(mut network) = URL::from_str("network://").open() {
            if url.path().len() > 0 {
                let ethertype = url.path().to_num_radix(16) as u16;

                if url.host().len() > 0 {
                    return Some(box EthernetResource {
                        network: network,
                        data: Vec::new(),
                        peer_addr: MACAddr::from_string(&url.host()),
                        ethertype: ethertype,
                    });
                } else {
                    loop {
                        let mut bytes: Vec<u8> = Vec::new();
                        match network.read_to_end(&mut bytes) {
                            Some(_) => {
                                if let Some(frame) = EthernetII::from_bytes(bytes) {
                                    if frame.header.ethertype.get() == ethertype &&
                                       (unsafe { frame.header.dst.equals(MAC_ADDR) } ||
                                        frame.header.dst.equals(BROADCAST_MAC_ADDR)) {
                                        return Some(box EthernetResource {
                                            network: network,
                                            data: frame.data,
                                            peer_addr: frame.header.src,
                                            ethertype: ethertype,
                                        });
                                    }
                                }
                            }
                            None => break,
                        }
                    }
                }
            } else {
                debug::d("Ethernet: No ethertype provided\n");
            }
        }

        None
    }
}
