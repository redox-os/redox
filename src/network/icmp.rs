use core::clone::Clone;
use core::mem::size_of;
use core::option::Option;

use common::debug::*;
use common::vector::*;

use network::common::*;

#[derive(Copy, Clone)]
pub struct ICMPHeader {
    pub _type: u8,
    pub code: u8,
    pub checksum: Checksum,
    pub data: [u8; 4]
}

pub struct ICMP {
    header: ICMPHeader,
    data: Vector<u8>
}

impl FromBytes for ICMP {
    fn from_bytes(bytes: Vector<u8>) -> Option<ICMP> {
        if bytes.len() >= size_of::<ICMPHeader>() {
            unsafe {
                return Option::Some(ICMP {
                    header: *(bytes.data as *const ICMPHeader),
                    data: bytes.sub(size_of::<ICMPHeader>(), bytes.len() - size_of::<ICMPHeader>())
                });
            }
        }
        return Option::None;
    }
}

impl ToBytes for ICMP {
    fn to_bytes(&self) -> Vector<u8> {
        unsafe{
            let header_ptr: *const ICMPHeader = &self.header;
            Vector::<u8>::from_raw(header_ptr as *const u8, size_of::<ICMPHeader>()) + self.data.clone()
        }
    }
}

impl Response for ICMP {
    fn respond(&self) -> Vector<Vector<u8>> {
        d("        ");
        self.d();
        dl();

        if self.header._type == 0x08 {
            d("            Echo Reply\n");

            let mut ret = ICMP {
                header: self.header,
                data: self.data.clone()
            };

            ret.header._type = 0x00;

            unsafe{
                ret.header.checksum.data = 0;

                let header_ptr: *const ICMPHeader = &ret.header;
                ret.header.checksum.data = Checksum::compile(
                    Checksum::sum(header_ptr as usize, size_of::<ICMPHeader>()) +
                    Checksum::sum(ret.data.data as usize, ret.data.len())
                );
            }

            return Vector::from_value(ret.to_bytes());
        }

        return Vector::new();
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
