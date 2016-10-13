use std::{str, u16};

use resource_scheme::ResourceScheme;
use syscall;
use syscall::error::{Error, Result, EACCES, ENOENT, EINVAL};
use syscall::flag::O_RDWR;

use common::{MacAddr, EthernetII};
use resource::EthernetResource;

pub struct EthernetScheme;

impl ResourceScheme<EthernetResource> for EthernetScheme {
    fn open_resource(&self, url: &[u8], _flags: usize, uid: u32, _gid: u32) -> Result<Box<EthernetResource>> {
        if uid == 0 {
            let path = try!(str::from_utf8(url).or(Err(Error::new(EINVAL))));
            let mut parts = path.split("/");
            if let Some(host_string) = parts.next() {
                if let Some(ethertype_string) = parts.next() {
                    if let Ok(network) = syscall::open("network:", O_RDWR) {
                        let ethertype = u16::from_str_radix(ethertype_string, 16).unwrap_or(0) as u16;

                        if !host_string.is_empty() {
                            return Ok(Box::new(EthernetResource {
                                network: network,
                                data: Vec::new(),
                                peer_addr: MacAddr::from_str(host_string),
                                ethertype: ethertype,
                            }));
                        } else {
                            loop {
                                let mut bytes = [0; 65536];
                                match syscall::read(network, &mut bytes) {
                                    Ok(count) => {
                                        if let Some(frame) = EthernetII::from_bytes(&bytes[..count]) {
                                            if frame.header.ethertype.get() == ethertype {
                                                return Ok(Box::new(EthernetResource {
                                                    network: network,
                                                    data: frame.data,
                                                    peer_addr: frame.header.src,
                                                    ethertype: ethertype,
                                                }));
                                            }
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                        }
                    } else {
                        println!("Ethernet: Failed to open network:");
                    }
                } else {
                    println!("Ethernet: No ethertype provided");
                }
            } else {
                println!("Ethernet: No host provided");
            }

            Err(Error::new(ENOENT))
        } else {
            Err(Error::new(EACCES))
        }
    }
}
