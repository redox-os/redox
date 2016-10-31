use std::collections::{BTreeMap, VecDeque};
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::{cmp, str, u16};

use netutils::{getcfg, MacAddr, EthernetII};
use syscall;
use syscall::error::{Error, Result, EACCES, EBADF, EINVAL, EIO, EWOULDBLOCK};
use syscall::flag::O_NONBLOCK;
use syscall::scheme::SchemeMut;

#[derive(Clone)]
pub struct Handle {
    /// The flags this handle was opened with
    flags: usize,
    /// The Host's MAC address
    pub host_addr: MacAddr,
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
        let mut total = 0;
        loop {
            let mut bytes = [0; 65536];
            let count = self.network.read(&mut bytes)?;
            if count == 0 {
                break;
            }
            if let Some(frame) = EthernetII::from_bytes(&bytes[.. count]) {
                for (_id, handle) in self.handles.iter_mut() {
                    if frame.header.ethertype.get() == handle.ethertype {
                        handle.frames.push_back(frame.clone());
                    }
                }
                total += count;
            }
        }
        Ok(total)
    }
}

impl SchemeMut for EthernetScheme {
    fn open(&mut self, url: &[u8], flags: usize, uid: u32, _gid: u32) -> Result<usize> {
        if uid == 0 {
            let mac_addr = MacAddr::from_str(&getcfg("mac").map_err(|err| Error::new(err.raw_os_error().unwrap_or(EIO)))?);
            let path = try!(str::from_utf8(url).or(Err(Error::new(EINVAL))));

            let ethertype = u16::from_str_radix(path, 16).unwrap_or(0);

            let next_id = self.next_id;
            self.next_id += 1;

            self.handles.insert(next_id, Handle {
                flags: flags,
                host_addr: mac_addr,
                ethertype: ethertype,
                frames: VecDeque::new()
            });

            Ok(next_id)
        } else {
            Err(Error::new(EACCES))
        }
    }

    fn dup(&mut self, id: usize, _buf: &[u8]) -> Result<usize> {
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
            let data = frame.to_bytes();
            for (b, d) in buf.iter_mut().zip(data.iter()) {
                *b = *d;
            }

            Ok(cmp::min(buf.len(), data.len()))
        } else if handle.flags & O_NONBLOCK == O_NONBLOCK {
            Ok(0)
        } else {
            Err(Error::new(EWOULDBLOCK))
        }
    }

    fn write(&mut self, id: usize, buf: &[u8]) -> Result<usize> {
        let handle = self.handles.get(&id).ok_or(Error::new(EBADF))?;

        if let Some(mut frame) = EthernetII::from_bytes(buf) {
            frame.header.src = handle.host_addr;
            frame.header.ethertype.set(handle.ethertype);
            self.network.write(&frame.to_bytes()).map_err(|err| Error::new(err.raw_os_error().unwrap_or(EIO)))
        } else {
            Err(Error::new(EINVAL))
        }
    }

    fn fevent(&mut self, id: usize, _flags: usize) -> Result<usize> {
        let _handle = self.handles.get(&id).ok_or(Error::new(EBADF))?;

        Ok(id)
    }

    fn fpath(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let handle = self.handles.get(&id).ok_or(Error::new(EBADF))?;

        let path_string = format!("ethernet:{:X}", handle.ethertype);
        let path = path_string.as_bytes();

        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }

        Ok(i)
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
