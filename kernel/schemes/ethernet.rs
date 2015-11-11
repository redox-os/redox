use alloc::boxed::Box;

use collections::string::ToString;
use collections::vec::Vec;

use core::mem;

use common::debug;
use common::to_num::ToNum;
use common::parse_ip::*;
use common::parse_path::*;

use network::common::*;
use network::ethernet::*;

use schemes::{KScheme, Resource, Url};

/// A ethernet resource
pub struct EthernetResource {
    /// The network
    network: Box<Resource>,
    /// The data
    data: Vec<u8>,
    /// The MAC addresss
    peer_addr: MacAddr,
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

    fn url(&self) -> Url {
        Url::from_string(format!("ethernet:{}/{:X}", self.peer_addr.to_string(), self.ethertype))
    }

    fn read(&mut self, _: &mut [u8]) -> Option<usize> {
        debug::d("TODO: Implement read for ethernet:\n");
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
        let data = Vec::from(buf);

        match self.network.write(& EthernetII {
            header: EthernetIIHeader {
                src: unsafe { MAC_ADDR },
                dst: self.peer_addr,
                ethertype: n16::new(self.ethertype),
            },
            data: data,
        }.to_bytes()) {
            Some(_) => Some(buf.len()),
            None => None,
        }
    }

    fn sync(&mut self) -> bool {
        self.network.sync()
    }
}

pub struct EthernetScheme;

impl KScheme for EthernetScheme {
    fn scheme(&self) -> &str {
        "ethernet"
    }

    fn open(&mut self, url: &Url, _: usize) -> Option<Box<Resource>> {
        let parts: Vec<&str> = url.reference().split("/").collect();
        if let Some(host_string) = parts.get(0) {
            if let Some(ethertype_string) = parts.get(1) {
                if let Some(mut network) = Url::from_str("network:").open() {
                    let ethertype = ethertype_string.to_num_radix(16) as u16;

                    if ! host_string.is_empty() {
                        return Some(box EthernetResource {
                            network: network,
                            data: Vec::new(),
                            peer_addr: MacAddr::from_str(host_string),
                            ethertype: ethertype,
                        });
                    } else {
                        loop {
                            let mut bytes: Vec<u8> = Vec::new();
                            match network.read_to_end(&mut bytes) {
                                Some(count) => {
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
                    debug::d("Ethernet: Failed to open network:\n");
                }
            } else {
                debug::d("Ethernet: No ethertype provided\n");
            }
        } else {
            debug::d("Ethernet: No host provided\n");
        }

        None
    }
}
