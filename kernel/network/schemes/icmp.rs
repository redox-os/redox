use common::slice::GetSlice;

use collections::vec::Vec;

use core::{mem, slice};

use arch::context::context_switch;

use network::common::*;

use fs::{KScheme, Url};

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct IcmpHeader {
    pub _type: u8,
    pub code: u8,
    pub checksum: Checksum,
    pub data: [u8; 4],
}

pub struct Icmp {
    pub header: IcmpHeader,
    pub data: Vec<u8>,
}

impl FromBytes for Icmp {
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() >= mem::size_of::<IcmpHeader>() {
            unsafe {
                return Some(Icmp {
                    header: *(bytes.as_ptr() as *const IcmpHeader),
                    data: bytes.get_slice(mem::size_of::<IcmpHeader>()..).to_vec(),
                });
            }
        }
        None
    }
}

impl ToBytes for Icmp {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let header_ptr: *const IcmpHeader = &self.header;
            let mut ret = Vec::from(slice::from_raw_parts(header_ptr as *const u8,
                                                          mem::size_of::<IcmpHeader>()));
            ret.extend_from_slice(&self.data);
            ret
        }
    }
}

pub struct IcmpScheme;

impl KScheme for IcmpScheme {
    fn scheme(&self) -> &str {
        "icmp"
    }
}

impl IcmpScheme {
    pub fn reply_loop() {
        while let Ok(mut ip) = Url::from_str("ip:/1").unwrap().open() {
            loop {
                let mut bytes = [0; 8192];
                if let Ok(count) = ip.read(&mut bytes) {
                    if let Some(message) = Icmp::from_bytes(bytes[..count].to_vec()) {
                        if message.header._type == 0x08 {
                            let mut response = Icmp {
                                header: message.header,
                                data: message.data,
                            };

                            response.header._type = 0x00;

                            unsafe {
                                response.header.checksum.data = 0;

                                let header_ptr: *const IcmpHeader = &response.header;
                                response.header.checksum.data = Checksum::compile(
                                    Checksum::sum(header_ptr as usize, mem::size_of::<IcmpHeader>()) +
                                    Checksum::sum(response.data.as_ptr() as usize, response.data.len())
                                );
                            }

                            let _ = ip.write(&response.to_bytes());
                        }
                    }
                } else {
                    break;
                }
            }
            unsafe { context_switch() };
        }
    }
}
