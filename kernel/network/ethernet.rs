use collections::vec::Vec;

use core::mem;

use common::debug;

use network::common::*;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct EthernetIIHeader {
    pub dst: MACAddr,
    pub src: MACAddr,
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
                    data: bytes.sub(mem::size_of::<EthernetIIHeader>(),
                                    bytes.len() - mem::size_of::<EthernetIIHeader>()),
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
            let mut ret = Vec::from_raw_buf(header_ptr as *const u8,
                                            mem::size_of::<EthernetIIHeader>());
            ret.push_all(&self.data);
            ret
        }
    }
}

impl EthernetII {
    pub fn d(&self) {
        debug::d("Ethernet II ");
        debug::dh(self.header.ethertype.get() as usize);
        debug::d(" from ");
        self.header.src.d();
        debug::d(" to ");
        self.header.dst.d();
        debug::d(" data ");
        debug::dd(self.data.len());
    }
}
