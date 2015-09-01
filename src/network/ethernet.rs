use core::clone::Clone;
use core::mem::size_of;
use core::option::Option;

use common::debug::*;
use common::vec::*;

use network::arp::*;
use network::common::*;
use network::ipv4::*;

#[derive(Copy, Clone)]
pub struct EthernetIIHeader {
    pub dst: MACAddr,
    pub src: MACAddr,
    pub _type: n16
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
        return Option::None;
    }
}

impl ToBytes for EthernetII {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe{
            let header_ptr: *const EthernetIIHeader = &self.header;
            let mut ret = Vec::from_raw_buf(header_ptr as *const u8, size_of::<EthernetIIHeader>());
            ret.push_all(&self.data);
            return ret;
        }
    }
}

impl Response for EthernetII {
    fn respond(&self) -> Vec<Vec<u8>>{
        let mut ret: Vec<Vec<u8>> = Vec::new();
        if self.header.dst.equals(MAC_ADDR) || self.header.dst.equals(BROADCAST_MAC_ADDR) {
            if cfg!(debug_network){
                self.d();
                dl();
            }

            let responses: Vec<Vec<u8>>;
            match self.header._type.get() {
                0x0800 => match IPv4::from_bytes(self.data.clone()) {
                    Option::Some(packet) => responses = packet.respond(),
                    Option::None => responses = Vec::new()
                },
                0x0806 => match ARP::from_bytes(self.data.clone()) {
                    Option::Some(packet) => responses = packet.respond(),
                    Option::None => responses = Vec::new()
                },
                _ => responses = Vec::new()
            }

            for response in responses.iter() {
                ret.push(EthernetII {
                    header: EthernetIIHeader {
                        src: MAC_ADDR,
                        dst: self.header.src,
                        _type: self.header._type
                    },
                    data: response.clone()
                }.to_bytes());
            }
        }
        return ret;
    }
}

impl EthernetII {
    pub fn d(&self){
        d("Ethernet II ");
        dh(self.header._type.get() as usize);
        d(" from ");
        self.header.src.d();
        d(" to ");
        self.header.dst.d();
        d(" data ");
        dd(self.data.len());
    }
}
