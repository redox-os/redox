use std::{cmp, mem};

use resource_scheme::Resource;
use syscall;
use syscall::error::*;

use common::{n16, Ipv4Addr, Checksum, Ipv4Header, Ipv4, IP_ADDR, BROADCAST_IP_ADDR};

/// A IP (internet protocole) resource
pub struct IpResource {
    pub link: usize,
    pub data: Vec<u8>,
    pub peer_addr: Ipv4Addr,
    pub proto: u8,
    pub id: u16,
}

impl Resource for IpResource {
    fn dup(&self) -> Result<Box<Self>> {
        let link = try!(syscall::dup(self.link));
        Ok(Box::new(IpResource {
            link: link,
            data: self.data.clone(),
            peer_addr: self.peer_addr,
            proto: self.proto,
            id: self.id,
        }))
    }

    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let path_string = format!("ip:{}/{:X}", self.peer_addr.to_string(), self.proto);
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
        let count = try!(syscall::read(self.link, &mut bytes));

        if let Some(packet) = Ipv4::from_bytes(&bytes[..count]) {
            if packet.header.proto == self.proto &&
               (packet.header.dst.equals(unsafe { IP_ADDR }) || packet.header.dst.equals(BROADCAST_IP_ADDR)) &&
               (packet.header.src.equals(self.peer_addr) || self.peer_addr.equals(BROADCAST_IP_ADDR)) {
                for (b, d) in buf.iter_mut().zip(packet.data.iter()) {
                    *b = *d;
                }

                return Ok(cmp::min(buf.len(), packet.data.len()));
            }
        }

        Ok(0)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let ip_data = Vec::from(buf);

        self.id += 1;
        let mut ip = Ipv4 {
            header: Ipv4Header {
                ver_hlen: 0x40 | (mem::size_of::<Ipv4Header>() / 4 & 0xF) as u8, // No Options
                services: 0,
                len: n16::new((mem::size_of::<Ipv4Header>() + ip_data.len()) as u16), // No Options
                id: n16::new(self.id),
                flags_fragment: n16::new(0),
                ttl: 128,
                proto: self.proto,
                checksum: Checksum { data: 0 },
                src: unsafe { IP_ADDR },
                dst: self.peer_addr,
            },
            options: Vec::new(),
            data: ip_data,
        };

        unsafe {
            let header_ptr: *const Ipv4Header = &ip.header;
            ip.header.checksum.data =
                Checksum::compile(Checksum::sum(header_ptr as usize, mem::size_of::<Ipv4Header>()) +
                                  Checksum::sum(ip.options.as_ptr() as usize, ip.options.len()));
        }

        match syscall::write(self.link, &ip.to_bytes()) {
            Ok(_) => Ok(buf.len()),
            Err(err) => Err(err),
        }
    }

    fn sync(&mut self) -> Result<usize> {
        syscall::fsync(self.link)
    }
}

impl Drop for IpResource {
    fn drop(&mut self) {
        let _ = syscall::close(self.link);
    }
}
