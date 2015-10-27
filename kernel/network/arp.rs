use collections::slice;
use collections::vec::Vec;

use core::mem;

use network::common::*;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct ARPHeader {
    pub htype: n16,
    pub ptype: n16,
    pub hlen: u8,
    pub plen: u8,
    pub oper: n16,
    pub src_mac: MACAddr,
    pub src_ip: IPv4Addr,
    pub dst_mac: MACAddr,
    pub dst_ip: IPv4Addr,
}

pub struct ARP {
    pub header: ARPHeader,
    pub data: Vec<u8>,
}

impl FromBytes for ARP {
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() >= mem::size_of::<ARPHeader>() {
            unsafe {
                return Some(ARP {
                    header: *(bytes.as_ptr() as *const ARPHeader),
                    data: bytes[mem::size_of::<ARPHeader>() ..].to_vec()
                });
            }
        }
        None
    }
}

impl ToBytes for ARP {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let header_ptr: *const ARPHeader = &self.header;
            let mut ret = Vec::from(slice::from_raw_parts(header_ptr as *const u8, mem::size_of::<ARPHeader>()));
            ret.push_all(&self.data);
            ret
        }
    }
}
