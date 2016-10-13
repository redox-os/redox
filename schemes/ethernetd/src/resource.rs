use std::{cmp, mem};

use resource_scheme::Resource;
use syscall;
use syscall::error::*;

use common::{n16, MacAddr, EthernetIIHeader, EthernetII};

/// A ethernet resource
pub struct EthernetResource {
    /// The network
    pub network: usize,
    /// The data
    pub data: Vec<u8>,
    /// The MAC addresss
    pub peer_addr: MacAddr,
    /// The ethernet type
    pub ethertype: u16,
}

impl Resource for EthernetResource {
    fn dup(&self) -> Result<Box<Self>> {
        let network = try!(syscall::dup(self.network));
        Ok(Box::new(EthernetResource {
            network: network,
            data: self.data.clone(),
            peer_addr: self.peer_addr,
            ethertype: self.ethertype,
        }))
    }

    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let path_string = format!("ethernet:{}/{:X}", self.peer_addr.to_string(), self.ethertype);
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

        let mut bytes = [0; 65536];
        let count = try!(syscall::read(self.network, &mut bytes));

        if let Some(frame) = EthernetII::from_bytes(&bytes[..count]) {
            if frame.header.ethertype.get() == self.ethertype {
                for (b, d) in buf.iter_mut().zip(frame.data.iter()) {
                    *b = *d;
                }

                return Ok(cmp::min(buf.len(), frame.data.len()));
            }
        }

        Ok(0)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let data = Vec::from(buf);

        match syscall::write(self.network, &EthernetII {
                                      header: EthernetIIHeader {
                                          src: MacAddr { bytes: [0x50, 0x51, 0x52, 0x53, 0x54, 0x55] },
                                          dst: self.peer_addr,
                                          ethertype: n16::new(self.ethertype),
                                      },
                                      data: data,
                                  }
                                  .to_bytes()) {
            Ok(_) => Ok(buf.len()),
            Err(err) => Err(err),
        }
    }

    fn sync(&mut self) -> Result<usize> {
        syscall::fsync(self.network)
    }
}

impl Drop for EthernetResource {
    fn drop(&mut self) {
        let _ = syscall::close(self.network);
    }
}
