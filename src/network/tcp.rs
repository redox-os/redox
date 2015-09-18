use core::mem::size_of;
use core::option::Option;

use common::debug::*;
use common::vec::*;

use network::common::*;

#[derive(Copy, Clone)]
#[repr(packed)]
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
    pub header: TCPHeader,
    pub options: Vec<u8>,
    pub data: Vec<u8>
}

pub const TCP_FIN: u16 = 1;
pub const TCP_SYN: u16 = 1 << 1;
pub const TCP_RST: u16 = 1 << 2;
pub const TCP_PSH: u16 = 1 << 3;
pub const TCP_ACK: u16 = 1 << 4;

impl FromBytes for TCP {
    fn from_bytes(bytes: Vec<u8>) -> Option<TCP> {
        if bytes.len() >= size_of::<TCPHeader>() {
            unsafe {
                let header = *(bytes.as_ptr() as *const TCPHeader);
                let header_len = ((header.flags.get() & 0xF000) >> 10) as usize;

                return Option::Some(TCP {
                    header: header,
                    options: bytes.sub(size_of::<TCPHeader>(), header_len - size_of::<TCPHeader>()),
                    data: bytes.sub(header_len, bytes.len() - header_len)
                });
            }
        }
        return Option::None;
    }
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

impl TCP {
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
