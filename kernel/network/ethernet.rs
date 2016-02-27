use common::slice::GetSlice;

use collections::slice;
use collections::vec::Vec;

use core::mem;

use network::common::*;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct EthernetIIHeader {
    pub dst: MacAddr,
    pub src: MacAddr,
    pub ethertype: n16,
}

pub struct EthernetII {
    pub header: EthernetIIHeader,
    pub data: Vec<u8>,
}

impl FromBytes for EthernetII {
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() >= mem::size_of::<EthernetIIHeader>() {
            unsafe {
                return Some(EthernetII {
                    header: *(bytes.as_ptr() as *const EthernetIIHeader),
                    data: bytes.get_slice(mem::size_of::<EthernetIIHeader>() ..).to_vec(),
                });
            }
        }
        None
    }
}

impl ToBytes for EthernetII {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let header_ptr: *const EthernetIIHeader = &self.header;
            let mut ret = Vec::from(slice::from_raw_parts(header_ptr as *const u8,
                                                          mem::size_of::<EthernetIIHeader>()));
            ret.push_all(&self.data);
            ret
        }
    }
}
