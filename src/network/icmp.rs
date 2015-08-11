use core::clone::Clone;
use core::mem::size_of;
use core::option::Option;

use alloc::boxed::*;

use collections::vec::*;

use common::debug::*;

use network::common::*;

use programs::session::*;

#[derive(Copy, Clone)]
pub struct ICMPHeader {
    pub _type: u8,
    pub code: u8,
    pub checksum: Checksum,
    pub data: [u8; 4]
}

pub struct ICMP {
    header: ICMPHeader,
    data: Vec<u8>
}

impl FromBytes for ICMP {
    fn from_bytes(bytes: Vec<u8>) -> Option<ICMP> {
        if bytes.len() >= size_of::<ICMPHeader>() {
            unsafe {
                return Option::Some(ICMP {
                    header: *(bytes.as_ptr() as *const ICMPHeader),
                    data: Vec::from(&bytes[size_of::<ICMPHeader>() .. bytes.len() - size_of::<ICMPHeader>()])
                });
            }
        }
        return Option::None;
    }
}

impl ToBytes for ICMP {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe{
            let header_ptr: *const ICMPHeader = &self.header;
            let mut ret = Vec::from_raw_buf(header_ptr as *const u8, size_of::<ICMPHeader>());
            ret.push_all(&self.data);
            return ret;
        }
    }
}

impl Response for ICMP {
    #[allow(unused_variables)]
    fn respond(&self, session: &Session, callback: Box<FnBox(Vec<Vec<u8>>)>){
        if cfg!(debug_network){
            d("        ");
            self.d();
            dl();
        }

        if self.header._type == 0x08 {
            if cfg!(debug_network){
                d("            Echo Reply\n");
            }

            let mut response = ICMP {
                header: self.header,
                data: self.data.clone()
            };

            response.header._type = 0x00;

            unsafe{
                response.header.checksum.data = 0;

                let header_ptr: *const ICMPHeader = &response.header;
                response.header.checksum.data = Checksum::compile(
                    Checksum::sum(header_ptr as usize, size_of::<ICMPHeader>()) +
                    Checksum::sum(response.data.as_ptr() as usize, response.data.len())
                );
            }

            let mut ret: Vec<Vec<u8>> = Vec::new();
            ret.push(response.to_bytes());
            callback(ret);
        }
    }
}

impl ICMP {
    pub fn d(&self){
        d("ICMP ");
        dbh(self.header._type);
        d(" code ");
        dbh(self.header.code);
        d(" data ");
        dd(self.data.len());
    }
}
