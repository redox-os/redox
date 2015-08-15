use core::clone::Clone;
use core::mem::size_of;
use core::option::Option;

use alloc::boxed::*;

use common::debug::*;
use common::memory::*;
use common::random::*;
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

#[allow(trivial_casts)]
impl Response for TCP {
    fn respond(&self, callback: Box<FnBox(Vec<Vec<u8>>)>){
        if cfg!(debug_network){
            d("            ");
            self.d();
            dl();
        }

        let allow;
        match self.header.dst.get() {
            80 => allow = true,
            _ => allow = false
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

                unsafe{
                    response.header.checksum.data = 0;

                    let proto = n16::new(0x06);
                    let segment_len = n16::new((size_of::<TCPHeader>() + response.options.len() + response.data.len()) as u16);
                    response.header.checksum.data = Checksum::compile(
                        Checksum::sum((&response.src_ip as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                        Checksum::sum((&response.dst_ip as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                        Checksum::sum((&proto as *const n16) as usize, size_of::<n16>()) +
                        Checksum::sum((&segment_len as *const n16) as usize, size_of::<n16>()) +
                        Checksum::sum((&response.header as *const TCPHeader) as usize, size_of::<TCPHeader>()) +
                        Checksum::sum(response.options.as_ptr() as usize, response.options.len()) +
                        Checksum::sum(response.data.as_ptr() as usize, response.data.len())
                    );
                }

                let mut ret: Vec<Vec<u8>> = Vec::new();
                ret.push(response.to_bytes());
                callback(ret);
            }else if self.header.flags.get() & TCP_PSH != 0{
                if cfg!(debug_network){
                    d("            TCP PSH\n");
                }
                //Send TCP_ACK_PSH_FIN in one statement
                {
                    let tcp_header = self.header;
                    let tcp_options = self.options.clone();
                    let tcp_dst_ip = self.src_ip;
                    let tcp_data = self.data.clone();
                    let tcp_callback = box move |data: String|{
                        let mut response = TCP {
                            header: tcp_header,
                            options: tcp_options.clone(),
                            data: Vec::new(),
                            src_ip: IP_ADDR,
                            dst_ip: tcp_dst_ip
                        };

                        response.header.src = tcp_header.dst;
                        response.header.dst = tcp_header.src;
                        response.header.flags.set(tcp_header.flags.get() | TCP_FIN);
                        response.header.ack_num.set(tcp_header.sequence.get() + tcp_data.len() as u32);
                        response.header.sequence.set(tcp_header.ack_num.get());

                        unsafe{
                            let data_ptr = data.to_c_str();
                            response.data = Vec::from_raw_buf(data_ptr, data.len());
                            unalloc(data_ptr as usize);
                        }

                        response.header.checksum.data = 0;

                        let proto = n16::new(0x06);
                        let segment_len = n16::new((size_of::<TCPHeader>() + response.options.len() + response.data.len()) as u16);
                        unsafe{
                            response.header.checksum.data = Checksum::compile(
                                Checksum::sum((&response.src_ip as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                                Checksum::sum((&response.dst_ip as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                                Checksum::sum((&proto as *const n16) as usize, size_of::<n16>()) +
                                Checksum::sum((&segment_len as *const n16) as usize, size_of::<n16>()) +
                                Checksum::sum((&response.header as *const TCPHeader) as usize, size_of::<TCPHeader>()) +
                                Checksum::sum(response.options.as_ptr() as usize, response.options.len()) +
                                Checksum::sum(response.data.as_ptr() as usize, response.data.len())
                            );
                        }

                        let mut ret: Vec<Vec<u8>> = Vec::new();
                        ret.push(response.to_bytes());
                        callback(ret);
                    };

                    match self.header.dst.get() {
                        80 => {
                            let request = String::from_c_slice(self.data.as_slice());

                            let mut path = "/".to_string();

                            for row in request.split("\r\n".to_string()) {
                                let mut i = 0;
                                for col in row.split(" ".to_string()) {
                                    match i {
                                        1 => path = col,
                                        _ => ()
                                    }
                                    i += 1;
                                }
                                break;
                            }

                            //session.request(&URL::from_string("http://".to_string() + path), tcp_callback);
                        },
                        _ => ()
                    }
                }
            }
        }else{
            d("            TCP RST TODO\n");
        }
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
