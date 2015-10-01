use core::mem::size_of;
use core::option::Option;

use common::debug::*;
use common::vec::*;

use network::common::*;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct EthernetIIHeader {
    pub dst: MACAddr,
    pub src: MACAddr,
    pub ethertype: n16
}

pub struct EthernetII {
    pub header: EthernetIIHeader,
    pub data: Vec<u8>
}

impl FromBytes for EthernetII {
    fn from_bytes(bytes: Vec<u8>) -> Option<EthernetII> {
        if bytes.len() >= size_of::<EthernetIIHeader>() {
            unsafe {
                return Option::Some(EthernetII {
                    header: *(bytes.as_ptr() as *const EthernetIIHeader),
                    data: bytes.sub(size_of::<EthernetIIHeader>(), bytes.len() - size_of::<EthernetIIHeader>())
                });
            }
        }
        Option::None
    }
}

impl ToBytes for EthernetII {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let header_ptr: *const EthernetIIHeader = &self.header;
            let mut ret = Vec::from_raw_buf(header_ptr as *const u8, size_of::<EthernetIIHeader>());
            ret.push_all(&self.data);
            ret
        }
    }
}

impl EthernetII {
    pub fn d(&self) {
        d("Ethernet II ");
        dh(self.header.ethertype.get() as usize);
        d(" from ");
        self.header.src.d();
        d(" to ");
        self.header.dst.d();
        d(" data ");
        dd(self.data.len());
    }
}
