use core::mem;

use common::debug;
use common::vec::Vec;

use network::common::*;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct IPv4Header {
    pub ver_hlen: u8,
    pub services: u8,
    pub len: n16,
    pub id: n16,
    pub flags_fragment: n16,
    pub ttl: u8,
    pub proto: u8,
    pub checksum: Checksum,
    pub src: IPv4Addr,
    pub dst: IPv4Addr,
}

pub struct IPv4 {
    pub header: IPv4Header,
    pub options: Vec<u8>,
    pub data: Vec<u8>,
}

impl FromBytes for IPv4 {
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() >= mem::size_of::<IPv4Header>() {
            unsafe {
                let header = *(bytes.as_ptr() as *const IPv4Header);
                let header_len = ((header.ver_hlen & 0xF) << 2) as usize;

                return Some(IPv4 {
                    header: header,
                    options: bytes.sub(mem::size_of::<IPv4Header>(),
                                       header_len - mem::size_of::<IPv4Header>()),
                    data: bytes.sub(header_len, bytes.len() - header_len),
                });
            }
        }
        None
    }
}

impl ToBytes for IPv4 {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let header_ptr: *const IPv4Header = &self.header;
            let mut ret = Vec::<u8>::from_raw_buf(header_ptr as *const u8, mem::size_of::<IPv4Header>());
            ret.push_all(&self.options);
            ret.push_all(&self.data);
            ret
        }
    }
}

impl IPv4 {
    pub fn d(&self) {
        debug::d("IPv4 ");
        debug::dbh(self.header.proto);
        debug::d(" from ");
        self.header.src.d();
        debug::d(" to ");
        self.header.dst.d();
        debug::d(" options ");
        debug::dd(self.options.len());
        debug::d(" data ");
        debug::dd(self.data.len());
    }
}
