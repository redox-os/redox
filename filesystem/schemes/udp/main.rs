use std::fs::File;
use std::io::{Result, Read, Write, SeekFrom};
use std::mem;
use std::net::*;
use std::ptr;
use std::rand;
use std::slice;
use std::to_num::*;
use std::url::Url;

use system::error::{Error, ENOENT, ESPIPE};

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct UdpHeader {
    pub src: n16,
    pub dst: n16,
    pub len: n16,
    pub checksum: Checksum,
}

pub struct Udp {
    pub header: UdpHeader,
    pub data: Vec<u8>,
}

impl FromBytes for Udp {
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() >= mem::size_of::<UdpHeader>() {
            unsafe {
                Option::Some(Udp {
                    header: ptr::read(bytes.as_ptr() as *const UdpHeader),
                    data: bytes[mem::size_of::<UdpHeader>()..bytes.len()].to_vec(),
                })
            }
        } else {
            Option::None
        }
    }
}

impl ToBytes for Udp {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let header_ptr: *const UdpHeader = &self.header;
            let mut ret = Vec::from(slice::from_raw_parts(header_ptr as *const u8,
                                                          mem::size_of::<UdpHeader>()));
            ret.push_all(&self.data);
            ret
        }
    }
}

/// UDP resource
pub struct Resource {
    ip: File,
    data: Vec<u8>,
    peer_addr: IPv4Addr,
    peer_port: u16,
    host_port: u16,
}

impl Resource {
    pub fn dup(&self) -> Result<Box<Self>> {
        match self.ip.dup() {
            Ok(ip) => {
                Ok(box Resource {
                    ip: ip,
                    data: self.data.clone(),
                    peer_addr: self.peer_addr,
                    peer_port: self.peer_port,
                    host_port: self.host_port,
                })
            }
            Err(err) => Err(err),
        }
    }

    pub fn path(&self) -> Result<String> {
        Ok(format!("udp://{}:{}/{}",
                   self.peer_addr.to_string(),
                   self.peer_port,
                   self.host_port))
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if !self.data.is_empty() {
            let mut bytes: Vec<u8> = Vec::new();
            mem::swap(&mut self.data, &mut bytes);

            // TODO: Allow splitting
            let i = 0;
            while i < buf.len() && i < bytes.len() {
                buf[i] = bytes[i];
            }
            return Ok(i);
        }

        loop {
            let mut bytes: Vec<u8> = Vec::new();
            match self.ip.read_to_end(&mut bytes) {
                Ok(_) => {
                    if let Some(datagram) = Udp::from_bytes(bytes) {
                        if datagram.header.dst.get() == self.host_port &&
                           datagram.header.src.get() == self.peer_port {
                            // TODO: Allow splitting
                            let i = 0;
                            while i < buf.len() && i < datagram.data.len() {
                                buf[i] = datagram.data[i];
                            }
                            return Ok(i);
                        }
                    }
                }
                Err(err) => return Err(err),
            }
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let udp_data = Vec::from(buf);

        let mut udp = Udp {
            header: UdpHeader {
                src: n16::new(self.host_port),
                dst: n16::new(self.peer_port),
                len: n16::new((mem::size_of::<UdpHeader>() + udp_data.len()) as u16),
                checksum: Checksum { data: 0 },
            },
            data: udp_data,
        };

        unsafe {
            let proto = n16::new(0x11);
            let datagram_len = n16::new((mem::size_of::<UdpHeader>() + udp.data.len()) as u16);
            udp.header.checksum.data =
                Checksum::compile(Checksum::sum((&IP_ADDR as *const IPv4Addr) as usize,
                                                mem::size_of::<IPv4Addr>()) +
                                  Checksum::sum((&self.peer_addr as *const IPv4Addr) as usize,
                                                mem::size_of::<IPv4Addr>()) +
                                  Checksum::sum((&proto as *const n16) as usize,
                                                mem::size_of::<n16>()) +
                                  Checksum::sum((&datagram_len as *const n16) as usize,
                                                mem::size_of::<n16>()) +
                                  Checksum::sum((&udp.header as *const UdpHeader) as usize,
                                                mem::size_of::<UdpHeader>()) +
                                  Checksum::sum(udp.data.as_ptr() as usize, udp.data.len()));
        }

        match self.ip.write(&udp.to_bytes()) {
            Ok(_) => Ok(buf.len()),
            Err(err) => Err(err),
        }
    }

    pub fn seek(&mut self, _: SeekFrom) -> Result<u64> {
        Err(Error::new(ESPIPE))
    }

    pub fn sync(&mut self) -> Result<()> {
        self.ip.sync_all()
    }
}

/// UDP scheme
pub struct Scheme;

impl Scheme {
    pub fn new() -> Box<Self> {
        box Scheme
    }

    pub fn open(&mut self, url_str: &str, _: usize) -> Result<Box<Resource>> {
        let url = Url::from_str(&url_str);

        // Check host and port vs path
        if !url.path().is_empty() {
            let host_port = url.port().to_num();
            if host_port > 0 && host_port < 65536 {
                if let Ok(mut ip) = File::open("ip:///11") {
                    let mut bytes: Vec<u8> = Vec::new();
                    if let Ok(_) = ip.read_to_end(&mut bytes) {
                        if let Some(datagram) = Udp::from_bytes(bytes) {
                            if datagram.header.dst.get() as u32 == host_port {
                                if let Ok(path) = ip.path() {
                                    let url = Url::from_string(path.to_string());

                                    return Ok(box Resource {
                                        ip: ip,
                                        data: datagram.data,
                                        peer_addr: IPv4Addr::from_string(&url.host()),
                                        peer_port: datagram.header.src.get(),
                                        host_port: host_port as u16,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        } else {
            let peer_port = url.port().to_num();
            if peer_port > 0 && peer_port < 65536 {
                let host_port = (rand() % 32768 + 32768) as u16;

                if let Ok(ip) = File::open(&format!("ip://{}/11", url.host())) {
                    return Ok(box Resource {
                        ip: ip,
                        data: Vec::new(),
                        peer_addr: IPv4Addr::from_string(&url.host()),
                        peer_port: peer_port as u16,
                        host_port: host_port,
                    });
                }
            }
        }

        Err(Error::new(ENOENT))
    }
}
