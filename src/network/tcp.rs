use core::clone::Clone;
use core::mem::size_of;
use core::option::Option;

use common::debug::*;
use common::net::*;
use common::random::*;
use common::resource::*;
use common::string::*;
use common::vec::*;

use network::common::*;

#[derive(Copy, Clone)]
pub struct TCPHeader {
    pub src: n16,
    pub dst: n16,
    pub sequence: n32,
    pub ack_num: n32,
    pub flags: n16,
    pub window_size: n16,
    pub checksum: Checksum,
    pub urgent_pointer: n16
}

pub struct TCP {
    header: TCPHeader,
    options: Vec<u8>,
    data: Vec<u8>,
    src_ip: IPv4Addr,
    dst_ip: IPv4Addr
}

impl ToBytes for TCP {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe{
            let header_ptr: *const TCPHeader = &self.header;
            let mut ret = Vec::from_raw_buf(header_ptr as *const u8, size_of::<TCPHeader>());
            ret.push_all(&self.options);
            ret.push_all(&self.data);
            return ret;
        }
    }
}

const TCP_FIN: u16 = 1;
const TCP_SYN: u16 = 1 << 1;
const TCP_RST: u16 = 1 << 2;
const TCP_PSH: u16 = 1 << 3;
const TCP_ACK: u16 = 1 << 4;

impl TCP {
    fn checksum(&mut self){
        self.header.checksum.data = 0;

        let proto = n16::new(0x06);
        let segment_len = n16::new((size_of::<TCPHeader>() + self.options.len() + self.data.len()) as u16);
        unsafe{
            self.header.checksum.data = Checksum::compile(
                Checksum::sum((&self.src_ip as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                Checksum::sum((&self.dst_ip as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                Checksum::sum((&proto as *const n16) as usize, size_of::<n16>()) +
                Checksum::sum((&segment_len as *const n16) as usize, size_of::<n16>()) +
                Checksum::sum((&self.header as *const TCPHeader) as usize, size_of::<TCPHeader>()) +
                Checksum::sum(self.options.as_ptr() as usize, self.options.len()) +
                Checksum::sum(self.data.as_ptr() as usize, self.data.len())
            );
        }
    }
}

#[allow(trivial_casts)]
impl Response for TCP {
    fn respond(&self) -> Vec<Vec<u8>>{
        if cfg!(debug_network){
            d("            ");
            self.d();
            dl();
        }

        let mut ret: Vec<Vec<u8>> = Vec::new();

        let mut allow = false;
        unsafe{
            for listener_ptr in (*::session_ptr).tcp_listeners.iter() {
                let listener = &mut **listener_ptr;
                if listener.port == self.header.dst.get() {
                    allow = true;
                    break;
                }
            }
        }

        if allow {
            if self.header.flags.get() & TCP_SYN != 0 {
                if cfg!(debug_network){
                    d("            TCP SYN\n");
                }
                let mut response = TCP {
                    header: self.header,
                    options: self.options.clone(),
                    data: Vec::new(),
                    src_ip: IP_ADDR,
                    dst_ip: self.src_ip
                };

                response.header.src = self.header.dst;
                response.header.dst = self.header.src;
                response.header.flags.set(self.header.flags.get() | TCP_ACK);
                response.header.ack_num.set(self.header.sequence.get() + 1);
                response.header.sequence.set(rand() as u32);

                response.checksum();

                ret.push(response.to_bytes());
            }else if self.header.flags.get() & TCP_PSH != 0{
                if cfg!(debug_network){
                    d("            TCP PSH\n");
                }

                unsafe{
                    for listener_ptr in (*::session_ptr).tcp_listeners.iter() {
                        let listener = &mut **listener_ptr;
                        if listener.port == self.header.dst.get() {
                            listener.streams.push(box TcpStream {
                                address: self.src_ip,
                                port: self.header.src.get(),
                                data: self.data.clone(),
                                response: Vec::new()
                            });
                            break;
                        }
                    }
                }

                let mut response = TCP {
                    header: self.header,
                    options: self.options.clone(),
                    data: Vec::new(),
                    src_ip: IP_ADDR,
                    dst_ip: self.src_ip
                };

                response.header.src = self.header.dst;
                response.header.dst = self.header.src;
                response.header.flags.set(self.header.flags.get() | TCP_FIN);
                response.header.ack_num.set(self.header.sequence.get() + self.data.len() as u32);
                response.header.sequence.set(self.header.ack_num.get());

                response.checksum();

                ret.push(response.to_bytes());
            }
        }else{
            if self.header.flags.get() & TCP_SYN != 0 {
                if cfg!(debug_network){
                    d("            TCP RST\n");
                }
                let mut response = TCP {
                    header: self.header,
                    options: self.options.clone(),
                    data: Vec::new(),
                    src_ip: IP_ADDR,
                    dst_ip: self.src_ip
                };

                response.header.src = self.header.dst;
                response.header.dst = self.header.src;
                response.header.flags.set(self.header.flags.get() | TCP_ACK | TCP_RST | TCP_FIN);
                response.header.ack_num.set(self.header.sequence.get() + 1);
                response.header.sequence.set(rand() as u32);

                response.checksum();

                ret.push(response.to_bytes());
            }
        }

        return ret;
    }
}

impl TCP {
    pub fn from_bytes_ipv4(bytes: Vec<u8>, src_ip: IPv4Addr, dst_ip: IPv4Addr) -> Option<TCP> {
        if bytes.len() >= size_of::<TCPHeader>() {
            unsafe {
                let header = *(bytes.as_ptr() as *const TCPHeader);
                let header_len = ((header.flags.get() & 0xF000) >> 10) as usize;

                return Option::Some(TCP {
                    header: header,
                    options: bytes.sub(size_of::<TCPHeader>(), header_len - size_of::<TCPHeader>()),
                    data: bytes.sub(header_len, bytes.len() - header_len),
                    src_ip: src_ip,
                    dst_ip: dst_ip
                });
            }
        }
        return Option::None;
    }

    pub fn d(&self){
        d("TCP from ");
        dd(self.header.src.get() as usize);
        d(" to ");
        dd(self.header.dst.get() as usize);
        d(" options ");
        dd(self.options.len());
        d(" data ");
        dd(self.data.len());
    }
}
