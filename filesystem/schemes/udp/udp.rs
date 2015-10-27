use redox::Box;
use redox::fs::File;
use redox::io::{Read, Write, SeekFrom};
use redox::mem;
use redox::net::*;
use redox::ptr;
use redox::rand;
use redox::slice;
use redox::{String, ToString};
use redox::to_num::*;
use redox::Vec;
use redox::URL;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct UDPHeader {
    pub src: n16,
    pub dst: n16,
    pub len: n16,
    pub checksum: Checksum,
}

pub struct UDP {
    pub header: UDPHeader,
    pub data: Vec<u8>,
}

impl FromBytes for UDP {
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() >= mem::size_of::<UDPHeader>() {
            unsafe {
                return Option::Some(UDP {
                    header: ptr::read(bytes.as_ptr() as *const UDPHeader),
                    data: bytes[mem::size_of::<UDPHeader>().. bytes.len()].to_vec(),
                });
            }
        }
        Option::None
    }
}

impl ToBytes for UDP {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let header_ptr: *const UDPHeader = &self.header;
            let mut ret = Vec::from(slice::from_raw_parts(header_ptr as *const u8, mem::size_of::<UDPHeader>()));
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
    pub fn dup(&self) -> Option<Box<Self>> {
        match self.ip.dup() {
            Some(ip) => Some(box Resource {
                ip: ip,
                data: self.data.clone(),
                peer_addr: self.peer_addr,
                peer_port: self.peer_port,
                host_port: self.host_port,
            }),
            None => None
        }
    }

    pub fn path(&self) -> Option<String> {
        Some(format!("udp://{}:{}/{}", self.peer_addr.to_string(), self.peer_port, self.host_port))
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        if self.data.len() > 0 {
            let mut bytes: Vec<u8> = Vec::new();
            mem::swap(&mut self.data, &mut bytes);

            //TODO: Allow splitting
            let i = 0;
            while i < buf.len() && i < bytes.len() {
                 buf[i] = bytes[i];
            }
            return Some(i);
        }

        loop {
            let mut bytes: Vec<u8> = Vec::new();
            match self.ip.read_to_end(&mut bytes) {
                Some(_) => {
                    if let Some(datagram) = UDP::from_bytes(bytes) {
                        if datagram.header.dst.get() == self.host_port &&
                           datagram.header.src.get() == self.peer_port {
                            //TODO: Allow splitting
                            let i = 0;
                            while i < buf.len() && i < datagram.data.len() {
                                buf[i] = datagram.data[i];
                            }
                            return Some(i);
                        }
                    }
                }
                None => break,
            }
        }

        None
    }

    pub fn write(&mut self, buf: &[u8]) -> Option<usize> {
        let udp_data = Vec::from(buf);

        let mut udp = UDP {
            header: UDPHeader {
                src: n16::new(self.host_port),
                dst: n16::new(self.peer_port),
                len: n16::new((mem::size_of::<UDPHeader>() + udp_data.len()) as u16),
                checksum: Checksum { data: 0 },
            },
            data: udp_data,
        };

        unsafe {
            let proto = n16::new(0x11);
            let datagram_len = n16::new((mem::size_of::<UDPHeader>() + udp.data.len()) as u16);
            udp.header.checksum.data =
                Checksum::compile(Checksum::sum((&IP_ADDR as *const IPv4Addr) as usize,
                                                mem::size_of::<IPv4Addr>()) +
                                  Checksum::sum((&self.peer_addr as *const IPv4Addr) as usize,
                                                mem::size_of::<IPv4Addr>()) +
                                  Checksum::sum((&proto as *const n16) as usize,
                                                mem::size_of::<n16>()) +
                                  Checksum::sum((&datagram_len as *const n16) as usize,
                                                mem::size_of::<n16>()) +
                                  Checksum::sum((&udp.header as *const UDPHeader) as usize,
                                                mem::size_of::<UDPHeader>()) +
                                  Checksum::sum(udp.data.as_ptr() as usize, udp.data.len()));
        }

        match self.ip.write(udp.to_bytes().as_slice()) {
            Some(_) => return Some(buf.len()),
            None => return None,
        }
    }

    pub fn seek(&mut self, pos: SeekFrom) -> Option<usize> {
        None
    }

    pub fn sync(&mut self) -> bool {
        self.ip.sync()
    }
}

/// UDP scheme
pub struct Scheme;

impl Scheme {
    pub fn new() -> Box<Self> {
        box Scheme
    }

    pub fn open(&mut self, url_str: &str) -> Option<Box<Resource>> {
        let url = URL::from_str(&url_str);

        //Check host and port vs path
        if url.path().len() > 0 {
            let host_port = url.port().to_num();
            if host_port > 0 && host_port < 65536 {
                if let Some(mut ip) = File::open("ip:///11") {
                    let mut bytes: Vec<u8> = Vec::new();
                    if ip.read_to_end(&mut bytes).is_some() {
                        if let Some(datagram) = UDP::from_bytes(bytes) {
                            if datagram.header.dst.get() as usize == host_port {
                                if let Some(path) = ip.path() {
                                    let url = URL::from_string(&path);

                                    return Some(box Resource {
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

                if let Some(ip) = File::open(&format!("ip://{}/11", url.host())) {
                    return Some(box Resource {
                        ip: ip,
                        data: Vec::new(),
                        peer_addr: IPv4Addr::from_string(&url.host()),
                        peer_port: peer_port as u16,
                        host_port: host_port,
                    });
                }
            }
        }

        None
    }
}
