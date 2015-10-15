use redox::Box;
use redox::fs::file::File;
use redox::io::{Read, Write, Seek, SeekFrom};
use redox::mem;
use redox::net::*;
use redox::ptr;
use redox::rand;
use redox::slice;
use redox::{str, String, ToString};
use redox::to_num::*;
use redox::Vec;

/// Ethernet resource
pub struct Resource {
    /// The network
    network: Box<Resource>,
    /// The data
    data: Vec<u8>,
    /// The MAC addresss
    peer_addr: MACAddr,
    /// The ethernet type
    ethertype: u16,
}

impl Resource {
    fn dup(&self) -> Option<Box<Self>> {
        match self.network.dup() {
            Some(network) => Some(box Resource {
                network: network,
                data: self.data.clone(),
                peer_addr: self.peer_addr,
                ethertype: self.ethertype,
            }),
            None => None
        }
    }

    pub fn path(&self, buf: &mut [u8]) -> Option<usize> {
        let path = format!("ethernet://{}/{}", self.peer_addr.to_string(), String::from_num_radix(self.ethertype as usize, 16));

        let mut i = 0;
        for b in path.bytes() {
            if i < buf.len() {
                buf[i] = b;
                i += 1;
            } else {
                break;
            }
        }

        Some(i)
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        /*
        if self.data.len() > 0 {
            let mut bytes: Vec<u8> = Vec::new();
            swap(&mut self.data, &mut bytes);
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
        */
        None
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        let data = Vec::from(buf);

        /*
        match self.network.write(EthernetII {
            header: EthernetIIHeader {
                src: unsafe { MAC_ADDR },
                dst: self.peer_addr,
                ethertype: n16::new(self.ethertype),
            },
            data: data,
        }.to_bytes().as_slice()) {
            Some(_) => return Some(buf.len()),
            None => return None,
        }
        */
        None
    }

    fn seek(&mut self, pos: SeekFrom) -> Option<usize> {
        None
    }

    fn sync(&mut self) -> bool {
        self.network.sync()
    }
}

pub struct Scheme;

impl Scheme {
    fn new() -> Box<Self> {
        box Scheme
    }

    fn open(&mut self, url: &str) -> Option<Box<Resource>> {
        //Split scheme from the rest of the URL
        let (scheme, mut not_scheme) = url.split_at(url.find(':').unwrap_or(url.len()));

        //Remove the starting two slashes
        if not_scheme.starts_with("//") {
            not_scheme = &not_scheme[2..not_scheme.len() - 2];
        }

        //Check host and port vs path
        if not_scheme.starts_with("/") {
            if let Some(mut network) = File::open("network://") {
                if url.path().len() > 0 {
                    let ethertype = url.path().to_num_radix(16) as u16;

                    if url.host().len() > 0 {
                        return Some(box Resource {
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
                                            return Some(box Resource {
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
                    /*
                    debug::d("Ethernet: No ethertype provided\n");
                    */
                }
            }
        }

        None
    }
}
