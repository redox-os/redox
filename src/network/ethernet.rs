use core::clone::Clone;
use core::mem::size_of;
use core::option::Option;

use common::debug::*;
use common::vector::*;

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
    pub data: Vector<u8>
}

impl FromBytes for EthernetII {
    fn from_bytes(bytes: Vector<u8>) -> Option<EthernetII> {
        if bytes.len() >= size_of::<EthernetIIHeader>() {
            unsafe {
                return Option::Some(EthernetII {
                    header: *(bytes.data as *const EthernetIIHeader),
                    data: bytes.sub(size_of::<EthernetIIHeader>(), bytes.len() - size_of::<EthernetIIHeader>())
                });
            }
        }
        return Option::None;
    }
}

impl ToBytes for EthernetII {
    fn to_bytes(&self) -> Vector<u8> {
        unsafe{
            let header_ptr: *const EthernetIIHeader = &self.header;
            Vector::<u8>::from_raw(header_ptr as *const u8, size_of::<EthernetIIHeader>()) + self.data.clone()
        }
    }
}

impl Response for EthernetII {
    fn respond(&self) -> Vector<Vector<u8>> {
        if self.header.dst.equals(MAC_ADDR) || self.header.dst.equals(BROADCAST_MAC_ADDR) {
            if cfg!(debug_network){
                self.d();
                dl();
            }

            let mut responses: Vector<Vector<u8>> = Vector::new();
            match self.header._type.get() {
                0x0800 => match IPv4::from_bytes(self.data.clone()) {
                    Option::Some(packet) => responses = packet.respond(),
                    Option::None => ()
                },
                0x0806 => match ARP::from_bytes(self.data.clone()) {
                    Option::Some(packet) => responses = packet.respond(),
                    Option::None => ()
                },
                _ => ()
            }

            let mut ret: Vector<Vector<u8>> = Vector::new();
            for response in responses.as_slice() {
                ret = ret + Vector::from_value(EthernetII {
                    header: EthernetIIHeader {
                        src: MAC_ADDR,
                        dst: self.header.src,
                        _type: self.header._type
                    },
                    data: response.clone()
                }.to_bytes());
            }
            return ret;
        }

        Vector::<Vector<u8>>::new()
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
