use collections::vec::Vec;

use core::mem;

use common::debug;

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
                    data: bytes.sub(mem::size_of::<ARPHeader>(),
                                    bytes.len() - mem::size_of::<ARPHeader>()),
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
            let mut ret = Vec::from_raw_buf(header_ptr as *const u8, mem::size_of::<ARPHeader>());
            ret.push_all(&self.data);
            ret
        }
    }
}

impl ARP {
    pub fn d(&self) {
        debug::d("ARP hw ");
        debug::dh(self.header.htype.get() as usize);
        debug::d("#");
        debug::dd(self.header.hlen as usize);
        debug::d(" proto ");
        debug::dh(self.header.ptype.get() as usize);
        debug::d("#");
        debug::dd(self.header.plen as usize);
        debug::d(" oper ");
        debug::dh(self.header.oper.get() as usize);
        debug::d(" from ");
        self.header.src_mac.d();
        debug::d(" (");
        self.header.src_ip.d();
        debug::d(") to ");
        self.header.dst_mac.d();
        debug::d(" (");
        self.header.dst_ip.d();
        debug::d(") data ");
        debug::dd(self.data.len());
    }
}
