use core::clone::Clone;
use core::mem::size_of;
use core::option::Option;

use common::debug::*;
use common::net::*;
use common::vec::*;

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
    options: Vec<u8>,
    data: Vec<u8>
}

impl FromBytes for IPv4 {
    fn from_bytes(bytes: Vec<u8>) -> Option<IPv4> {
        if bytes.len() >= size_of::<IPv4Header>() {
            unsafe {
                let header = *(bytes.as_ptr() as *const IPv4Header);
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
    fn to_bytes(&self) -> Vec<u8> {
        unsafe{
            let header_ptr: *const IPv4Header = &self.header;
            let mut ret = Vec::<u8>::from_raw_buf(header_ptr as *const u8, size_of::<IPv4Header>());
            ret.push_all(&self.options);
            ret.push_all(&self.data);
            return ret;
        }
    }
}

impl Response for IPv4 {
    fn respond(&self) -> Vec<Vec<u8>> {
        let mut ret: Vec<Vec<u8>> = Vec::new();
        if self.header.dst.equals(IP_ADDR) || self.header.dst.equals(BROADCAST_IP_ADDR){
            if cfg!(debug_network){
                d("    ");
                self.d();
                dl();
            }

            let responses: Vec<Vec<u8>>;
            match self.header.proto {
                0x01 => match ICMP::from_bytes(self.data.clone()) {
                    Option::Some(packet) => responses = packet.respond(),
                    Option::None => responses = Vec::new()
                },
                //Must copy source IP and destination IP for checksum
                0x06 => match TCP::from_bytes_ipv4(self.data.clone(), self.header.src, self.header.dst) {
                    Option::Some(packet) => responses = packet.respond(),
                    Option::None => responses = Vec::new()
                },
                0x11 => match UDP::from_bytes(self.data.clone()) {
                    Option::Some(packet) => responses = packet.respond(),
                    Option::None => responses = Vec::new()
                },
                _ => responses = Vec::new()
            }

            for response in responses.iter() {
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
                        Checksum::sum(packet.options.as_ptr() as usize, packet.options.len())
                    );
                }

                ret.push(packet.to_bytes());
            }
        }
        return ret;
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
