use core::clone::Clone;
use core::mem::size_of;
use core::option::Option;

use alloc::boxed::*;

use common::debug::*;
use common::vec::*;

use network::common::*;
use network::icmp::*;
use network::tcp::*;
use network::udp::*;

use programs::session::*;

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
                    options: Vec::from(&bytes[size_of::<IPv4Header>() .. header_len - size_of::<IPv4Header>()]),
                    data: Vec::from(&bytes[header_len .. bytes.len() - header_len])
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
    fn respond(&self, session: &Session, callback: Box<FnBox(Vec<Vec<u8>>)>){
        if self.header.dst.equals(IP_ADDR) || self.header.dst.equals(BROADCAST_IP_ADDR){
            if cfg!(debug_network){
                d("    ");
                self.d();
                dl();
            }

            let ipv4_header = self.header;
            let ipv4_options = self.options.clone();
            let ipv4_callback = box move |responses: Vec<Vec<u8>>|{
                let mut ret: Vec<Vec<u8>> = Vec::new();
                for response in responses.iter() {
                    let mut packet = IPv4 {
                        header: ipv4_header,
                        options: ipv4_options.clone(),
                        data: response.clone()
                    };

                    packet.header.dst = ipv4_header.src;
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
                callback(ret);
            };

            match self.header.proto {
                0x01 => match ICMP::from_bytes(self.data.clone()) {
                    Option::Some(packet) => packet.respond(session, ipv4_callback),
                    Option::None => ()
                },
                //Must copy source IP and destination IP for checksum
                0x06 => match TCP::from_bytes_ipv4(self.data.clone(), self.header.src, self.header.dst) {
                    Option::Some(packet) => packet.respond(session, ipv4_callback),
                    Option::None => ()
                },
                0x11 => match UDP::from_bytes(self.data.clone()) {
                    Option::Some(packet) => packet.respond(session, ipv4_callback),
                    Option::None => ()
                },
                _ => ()
            }
        }
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
