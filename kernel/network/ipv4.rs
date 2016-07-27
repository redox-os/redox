use common::slice::GetSlice;

use collections::slice;
use collections::vec::Vec;

use core::mem;

use network::common::*;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Ipv4Header {
    pub ver_hlen: u8,
    pub services: u8,
    pub len: n16,
    pub id: n16,
    pub flags_fragment: n16,
    pub ttl: u8,
    pub proto: u8,
    pub checksum: Checksum,
    pub src: Ipv4Addr,
    pub dst: Ipv4Addr,
}

pub struct Ipv4 {
    pub header: Ipv4Header,
    pub options: Vec<u8>,
    pub data: Vec<u8>,
}

impl FromBytes for Ipv4 {
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() >= mem::size_of::<Ipv4Header>() {
            unsafe {
                let header = *(bytes.as_ptr() as *const Ipv4Header);
                let header_len = ((header.ver_hlen & 0xF) << 2) as usize;

                return Some(Ipv4 {
                    header: header,
                    options: bytes.get_slice(mem::size_of::<Ipv4Header>() .. header_len).to_vec(),
                    data: bytes.get_slice(header_len .. header.len.get() as usize).to_vec(),
                });
            }
        }
        None
    }
}

impl ToBytes for Ipv4 {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let header_ptr: *const Ipv4Header = &self.header;
            let mut ret = Vec::<u8>::from(slice::from_raw_parts(header_ptr as *const u8,
                                                                mem::size_of::<Ipv4Header>()));
            ret.extend_from_slice(&self.options);
            ret.extend_from_slice(&self.data);
            ret
        }
    }
}
