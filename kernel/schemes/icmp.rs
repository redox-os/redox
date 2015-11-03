use collections::vec::Vec;

use core::{mem, slice};

use scheduler::context::recursive_unsafe_yield;

use network::common::*;

use schemes::{KScheme, URL};

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct ICMPHeader {
    pub _type: u8,
    pub code: u8,
    pub checksum: Checksum,
    pub data: [u8; 4],
}

pub struct ICMP {
    pub header: ICMPHeader,
    pub data: Vec<u8>,
}

impl FromBytes for ICMP {
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() >= mem::size_of::<ICMPHeader>() {
            unsafe {
                return Some(ICMP {
                    header: *(bytes.as_ptr() as *const ICMPHeader),
                    data: bytes[mem::size_of::<ICMPHeader>()..].to_vec(),
                });
            }
        }
        None
    }
}

impl ToBytes for ICMP {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let header_ptr: *const ICMPHeader = &self.header;
            let mut ret = Vec::from(slice::from_raw_parts(header_ptr as *const u8, mem::size_of::<ICMPHeader>()));
            ret.push_all(&self.data);
            ret
        }
    }
}

pub struct ICMPScheme;

impl KScheme for ICMPScheme {
    fn scheme(&self) -> &str {
        "icmp"
    }
}

impl ICMPScheme {
    pub fn reply_loop() {
        while let Some(mut ip) = URL::from_str("ip:///1").open() {
            loop {
                let mut bytes: Vec<u8> = Vec::new();
                if let Some(_) = ip.read_to_end(&mut bytes) {
                    if let Some(message) = ICMP::from_bytes(bytes) {
                        if message.header._type == 0x08 {
                            let mut response = ICMP {
                                header: message.header,
                                data: message.data,
                            };

                            response.header._type = 0x00;

                            unsafe {
                                response.header.checksum.data = 0;

                                let header_ptr: *const ICMPHeader = &response.header;
                                response.header.checksum.data = Checksum::compile(
                                    Checksum::sum(header_ptr as usize, mem::size_of::<ICMPHeader>()) +
                                    Checksum::sum(response.data.as_ptr() as usize, response.data.len())
                                );
                            }

                            ip.write(&response.to_bytes());
                        }
                    }
                } else {
                    break;
                }
            }
            unsafe { recursive_unsafe_yield() };
        }
    }
}
