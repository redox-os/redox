use core::clone::Clone;
use core::mem::size_of;
use core::option::Option;

use common::debug::*;
use common::vector::*;

use network::common::*;
use network::icmp::*;
use network::tcp::*;
use network::udp::*;

#[derive(Copy, Clone)]
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
    pub dst: IPv4Addr
}

pub struct IPv4 {
    header: IPv4Header,
    options: Vector<u8>,
    data: Vector<u8>
}

impl FromBytes for IPv4 {
    fn from_bytes(bytes: Vector<u8>) -> Option<IPv4> {
        if bytes.len() >= size_of::<IPv4Header>() {
            unsafe {
                let header = *(bytes.data as *const IPv4Header);
                let header_len = ((header.ver_hlen & 0xF) << 2) as usize;

                return Option::Some(IPv4 {
                    header: header,
                    options: bytes.sub(size_of::<IPv4Header>(), header_len - size_of::<IPv4Header>()),
                    data: bytes.sub(header_len, bytes.len() - header_len)
                });
            }
        }
        return Option::None;
    }
}

impl ToBytes for IPv4 {
    fn to_bytes(&self) -> Vector<u8> {
        unsafe{
            let header_ptr: *const IPv4Header = &self.header;
            Vector::<u8>::from_raw(header_ptr as *const u8, size_of::<IPv4Header>()) + self.options.clone() + self.data.clone()
        }
    }
}

impl Response for IPv4 {
    fn respond(&self) -> Vector<Vector<u8>>{
        if self.header.dst.equals(IP_ADDR) || self.header.dst.equals(BROADCAST_IP_ADDR){
            d("    ");
            self.d();
            dl();

            let mut responses: Vector<Vector<u8>> = Vector::new();
            match self.header.proto {
                0x01 => match ICMP::from_bytes(self.data.clone()) {
                    Option::Some(packet) => responses = packet.respond(),
                    Option::None => ()
                },
                0x06 => match TCP::from_bytes(self.data.clone()) {
                    Option::Some(packet) => responses = packet.respond(),
                    Option::None => ()
                },
                0x11 => match UDP::from_bytes(self.data.clone()) {
                    Option::Some(packet) => responses = packet.respond(),
                    Option::None => ()
                },
                _ => ()
            }

            let mut ret: Vector<Vector<u8>> = Vector::new();
            for response in responses.as_slice() {
                let mut packet = IPv4 {
                    header: self.header,
                    options: self.options.clone(),
                    data: response.clone()
                };

                packet.header.dst = self.header.src;
                packet.header.src = IP_ADDR;
                packet.header.len.set((size_of::<IPv4Header>() + packet.options.len() + packet.data.len()) as u16);

                unsafe{
                    packet.header.checksum.data = 0;

                    let header_ptr: *const IPv4Header = &packet.header;
                    packet.header.checksum.data = Checksum::compile(
                        Checksum::sum(header_ptr as usize, size_of::<IPv4Header>()) +
                        Checksum::sum(packet.options.data as usize, packet.options.len())
                    );
                }

                ret = ret + Vector::from_value(packet.to_bytes());
            }
            return ret;
        }

        return Vector::new();
    }
}

impl IPv4 {
    pub fn d(&self){
        d("IPv4 ");
        dbh(self.header.proto);
        d(" from ");
        self.header.src.d();
        d(" to ");
        self.header.dst.d();
        d(" options ");
        dd(self.options.len());
        d(" data ");
        dd(self.data.len());
    }
}