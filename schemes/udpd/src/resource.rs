use std::{cmp, mem};

use netutils::{n16, Ipv4Addr, Checksum, Udp, UdpHeader};
use resource_scheme::Resource;
use syscall;
use syscall::error::*;

/// UDP resource
pub struct UdpResource {
    pub ip: usize,
    pub data: Vec<u8>,
    pub host_addr: Ipv4Addr,
    pub peer_addr: Ipv4Addr,
    pub peer_port: u16,
    pub host_port: u16,
}

impl Resource for UdpResource {
    fn dup(&self) -> Result<Box<UdpResource>> {
        match syscall::dup(self.ip) {
            Ok(ip) => {
                Ok(Box::new(UdpResource {
                    ip: ip,
                    data: self.data.clone(),
                    host_addr: self.host_addr,
                    peer_addr: self.peer_addr,
                    peer_port: self.peer_port,
                    host_port: self.host_port,
                }))
            }
            Err(err) => Err(err),
        }
    }

    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let path_string = format!("udp:{}:{}/{}", self.peer_addr.to_string(), self.peer_port, self.host_port);
        let path = path_string.as_bytes();

        for (b, p) in buf.iter_mut().zip(path.iter()) {
            *b = *p;
        }

        Ok(cmp::min(buf.len(), path.len()))
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if ! self.data.is_empty() {
            let mut bytes: Vec<u8> = Vec::new();
            mem::swap(&mut self.data, &mut bytes);

            // TODO: Allow splitting
            let mut i = 0;
            while i < buf.len() && i < bytes.len() {
                buf[i] = bytes[i];
                i += 1;
            }

            return Ok(i);
        }

        loop {
            let mut bytes = [0; 65536];
            let count = try!(syscall::read(self.ip, &mut bytes));

            if let Some(datagram) = Udp::from_bytes(&bytes[..count]) {
                if datagram.header.dst.get() == self.host_port &&
                   datagram.header.src.get() == self.peer_port {
                    // TODO: Allow splitting
                    let mut i = 0;
                    while i < buf.len() && i < datagram.data.len() {
                        buf[i] = datagram.data[i];
                        i += 1;
                    }

                    return Ok(i);
                }
            }
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut udp = Udp {
            header: UdpHeader {
                src: n16::new(self.host_port),
                dst: n16::new(self.peer_port),
                len: n16::new((mem::size_of::<UdpHeader>() + buf.len()) as u16),
                checksum: Checksum { data: 0 },
            },
            data: Vec::from(buf),
        };

        unsafe {
            let proto = n16::new(0x11);
            let datagram_len = n16::new((mem::size_of::<UdpHeader>() + udp.data.len()) as u16);
            udp.header.checksum.data =
                Checksum::compile(Checksum::sum((&self.host_addr as *const Ipv4Addr) as usize,
                                                mem::size_of::<Ipv4Addr>()) +
                                  Checksum::sum((&self.peer_addr as *const Ipv4Addr) as usize,
                                                mem::size_of::<Ipv4Addr>()) +
                                  Checksum::sum((&proto as *const n16) as usize,
                                                mem::size_of::<n16>()) +
                                  Checksum::sum((&datagram_len as *const n16) as usize,
                                                mem::size_of::<n16>()) +
                                  Checksum::sum((&udp.header as *const UdpHeader) as usize,
                                                mem::size_of::<UdpHeader>()) +
                                  Checksum::sum(udp.data.as_ptr() as usize, udp.data.len()));
        }

        syscall::write(self.ip, &udp.to_bytes()).and(Ok(buf.len()))
    }

    fn sync(&mut self) -> Result<usize> {
        syscall::fsync(self.ip)
    }
}
