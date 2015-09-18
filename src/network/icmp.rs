use core::mem::size_of;
use core::option::Option;

use common::debug::*;
use common::vec::*;

use network::common::*;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct ICMPHeader {
    pub _type: u8,
    pub code: u8,
    pub checksum: Checksum,
    pub data: [u8; 4]
}

pub struct ICMP {
    pub header: ICMPHeader,
    pub data: Vec<u8>
}

impl FromBytes for ICMP {
    fn from_bytes(bytes: Vec<u8>) -> Option<ICMP> {
        if bytes.len() >= size_of::<ICMPHeader>() {
            unsafe {
                return Option::Some(ICMP {
                    header: *(bytes.as_ptr() as *const ICMPHeader),
                    data: bytes.sub(size_of::<ICMPHeader>(), bytes.len() - size_of::<ICMPHeader>())
                });
            }
        }
        return Option::None;
    }
}

impl ToBytes for ICMP {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe{
            let header_ptr: *const ICMPHeader = &self.header;
            let mut ret = Vec::from_raw_buf(header_ptr as *const u8, size_of::<ICMPHeader>());
            ret.push_all(&self.data);
            return ret;
        }
    }
}

impl ICMP {
    pub fn d(&self){
        d("ICMP ");
        dbh(self.header._type);
        d(" code ");
        dbh(self.header.code);
        d(" data ");
        dd(self.data.len());
    }
}
