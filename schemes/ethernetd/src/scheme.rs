use std::collections::{BTreeMap, VecDeque};
use std::fs::File;
use std::io::{self, Read};
use std::os::unix::io::AsRawFd;
use std::{cmp, str, u16};

use netutils::{getcfg, n16, MacAddr, EthernetII, EthernetIIHeader};
use syscall;
use syscall::error::{Error, Result, EACCES, EBADF, ENOENT, EINVAL, EWOULDBLOCK};
use syscall::flag::O_RDWR;
use syscall::scheme::SchemeMut;

#[derive(Clone)]
pub struct Handle {
    /// The Host's MAC address
    pub host_addr: MacAddr,
    /// The Peer's MAC address
    pub peer_addr: MacAddr,
    /// The ethernet type
    pub ethertype: u16,
    /// The data
    pub frames: VecDeque<EthernetII>,
}

pub struct EthernetScheme {
    network: File,
    next_id: usize,
    pub handles: BTreeMap<usize, Handle>
}

impl EthernetScheme {
    pub fn new(network: File) -> EthernetScheme {
        EthernetScheme {
            network: network,
            next_id: 1,
            handles: BTreeMap::new(),
        }
    }

    //TODO: Minimize allocation
    //TODO: Reduce iteration cost (use BTreeMap of ethertype to handle?)
    pub fn input(&mut self) -> io::Result<usize> {
        let mut bytes = [0; 65536];
        let count = self.network.read(&mut bytes)?;
        if let Some(frame) = EthernetII::from_bytes(&bytes[.. count]) {
            for (_id, handle) in self.handles.iter_mut() {
                if frame.header.ethertype.get() == handle.ethertype {
                    handle.frames.push_back(frame.clone());
                }
            }
            Ok(count)
        } else {
            Ok(0)
        }
    }
}

impl SchemeMut for EthernetScheme {
    fn open(&mut self, url: &[u8], _flags: usize, uid: u32, _gid: u32) -> Result<usize> {
        if uid == 0 {
            let mac_addr = MacAddr::from_str(&getcfg("mac").map_err(|err| err.into_sys())?);
            let path = try!(str::from_utf8(url).or(Err(Error::new(EINVAL))));
            let mut parts = path.split("/");
            if let Some(host_string) = parts.next() {
                if let Some(ethertype_string) = parts.next() {
                    if let Ok(network) = syscall::open("network:", O_RDWR) {
                        let ethertype = u16::from_str_radix(ethertype_string, 16).unwrap_or(0) as u16;

                        if ! host_string.is_empty() {
                            let next_id = self.next_id;
                            self.next_id += 1;

                            self.handles.insert(next_id, Handle {
                                host_addr: mac_addr,
                                peer_addr: MacAddr::from_str(host_string),
                                ethertype: ethertype,
                                frames: VecDeque::new()
                            });

                            return Ok(next_id);
                        } else {
                            loop {
                                let mut bytes = [0; 65536];
                                match syscall::read(network, &mut bytes) {
                                    Ok(count) => {
                                        if let Some(frame) = EthernetII::from_bytes(&bytes[..count]) {
                                            if frame.header.ethertype.get() == ethertype {
                                                let next_id = self.next_id;
                                                self.next_id += 1;

                                                let peer_addr = frame.header.src;

                                                let mut frames = VecDeque::new();
                                                frames.push_back(frame);

                                                self.handles.insert(next_id, Handle {
                                                    host_addr: mac_addr,
                                                    peer_addr: peer_addr,
                                                    ethertype: ethertype,
                                                    frames: frames
                                                });

                                                return Ok(next_id);
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

    fn dup(&mut self, id: usize) -> Result<usize> {
        let next_id = self.next_id;
        self.next_id += 1;

        let handle = {
            let handle = self.handles.get(&id).ok_or(Error::new(EBADF))?;
            handle.clone()
        };

        self.handles.insert(next_id, handle);

        Ok(next_id)
    }

    fn read(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let handle = self.handles.get_mut(&id).ok_or(Error::new(EBADF))?;

        if let Some(frame) = handle.frames.pop_front() {
            for (b, d) in buf.iter_mut().zip(frame.data.iter()) {
                *b = *d;
            }

            Ok(cmp::min(buf.len(), frame.data.len()))
        } else {
            Err(Error::new(EWOULDBLOCK))
        }
    }

    fn write(&mut self, id: usize, buf: &[u8]) -> Result<usize> {
        let handle = self.handles.get(&id).ok_or(Error::new(EBADF))?;

        match syscall::write(self.network.as_raw_fd(), &EthernetII {
                                      header: EthernetIIHeader {
                                          src: handle.host_addr,
                                          dst: handle.peer_addr,
                                          ethertype: n16::new(handle.ethertype),
                                      },
                                      data: Vec::from(buf),
                                  }
                                  .to_bytes()) {
            Ok(_) => Ok(buf.len()),
            Err(err) => Err(err),
        }
    }

    fn fevent(&mut self, id: usize, _flags: usize) -> Result<usize> {
        let _handle = self.handles.get(&id).ok_or(Error::new(EBADF))?;

        Ok(id)
    }

    fn fpath(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let handle = self.handles.get(&id).ok_or(Error::new(EBADF))?;

        let path_string = format!("ethernet:{}/{:X}", handle.peer_addr.to_string(), handle.ethertype);
        let path = path_string.as_bytes();

        for (b, p) in buf.iter_mut().zip(path.iter()) {
            *b = *p;
        }

        Ok(cmp::min(buf.len(), path.len()))
    }

    fn fsync(&mut self, id: usize) -> Result<usize> {
        let _handle = self.handles.get(&id).ok_or(Error::new(EBADF))?;

        syscall::fsync(self.network.as_raw_fd())
    }

    fn close(&mut self, id: usize) -> Result<usize> {
        let handle = self.handles.remove(&id).ok_or(Error::new(EBADF))?;
        drop(handle);
        Ok(0)
    }
}
