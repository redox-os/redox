use core::clone::Clone;
use core::mem::size_of;
use core::option::Option;

use common::debug::*;
use common::vector::*;

use network::common::*;

#[derive(Copy, Clone)]
pub struct ARPHeader {
    pub htype: n16,
    pub ptype: n16,
    pub hlen: u8,
    pub plen: u8,
    pub oper: n16,
    pub src_mac: MACAddr,
    pub src_ip: IPv4Addr,
    pub dst_mac: MACAddr,
    pub dst_ip: IPv4Addr
}

pub struct ARP {
    pub header: ARPHeader,
    pub data: Vector<u8>
}

impl FromBytes for ARP {
    fn from_bytes(bytes: Vector<u8>) -> Option<ARP> {
        if bytes.len() >= size_of::<ARPHeader>() {
            unsafe {
                return Option::Some(ARP {
                    header: *(bytes.data as *const ARPHeader),
                    data: bytes.sub(size_of::<ARPHeader>(), bytes.len() - size_of::<ARPHeader>())
                });
            }
        }
        return Option::None;
    }
}

impl ToBytes for ARP {
    fn to_bytes(&self) -> Vector<u8> {
        unsafe{
            let header_ptr: *const ARPHeader = &self.header;
            Vector::<u8>::from_raw(header_ptr as *const u8, size_of::<ARPHeader>()) + self.data.clone()
        }
    }
}

impl Response for ARP {
    fn respond(&self) -> Vector<Vector<u8>>{
        if self.header.dst_ip.equals(IP_ADDR) {
            d("    ");
            self.d();
            dl();

            if self.header.oper.get() == 1 {
                d("        ARP Reply\n");
                let mut response = ARP{
                    header: self.header,
                    data: self.data.clone()
                };
                response.header.oper.set(2);
                response.header.dst_mac = self.header.src_mac;
                response.header.dst_ip = self.header.src_ip;
                response.header.src_mac = MAC_ADDR;
                response.header.src_ip = IP_ADDR;

                return Vector::from_value(response.to_bytes());
            }
        }

        return Vector::new();
    }
}

impl ARP {
    pub fn d(&self){
        d("ARP hw ");
        dh(self.header.htype.get() as usize);
        d("#");
        dd(self.header.hlen as usize);
        d(" proto ");
        dh(self.header.ptype.get() as usize);
        d("#");
        dd(self.header.plen as usize);
        d(" oper ");
        dh(self.header.oper.get() as usize);
        d(" from ");
        self.header.src_mac.d();
        d(" (");
        self.header.src_ip.d();
        d(") to ");
        self.header.dst_mac.d();
        d(" (");
        self.header.dst_ip.d();
        d(") data ");
        dd(self.data.len());
    }
}