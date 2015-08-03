use core::clone::Clone;
use core::mem::size_of;
use core::option::Option;

use common::debug::*;
use common::vector::*;

use network::common::*;

use programs::session::*;

#[derive(Copy, Clone)]
pub struct UDPHeader {
    pub src: n16,
    pub dst: n16,
    pub len: n16,
    pub checksum: Checksum
}

pub struct UDP {
    header: UDPHeader,
    data: Vector<u8>
}

impl FromBytes for UDP {
    fn from_bytes(bytes: Vector<u8>) -> Option<UDP> {
        if bytes.len() >= size_of::<UDPHeader>() {
            unsafe {
                return Option::Some(UDP {
                    header: *(bytes.data as *const UDPHeader),
                    data: bytes.sub(size_of::<UDPHeader>(), bytes.len() - size_of::<UDPHeader>())
                });
            }
        }
        return Option::None;
    }
}

impl ToBytes for UDP {
    fn to_bytes(&self) -> Vector<u8> {
        unsafe{
            let header_ptr: *const UDPHeader = &self.header;
            return Vector::<u8>::from_raw(header_ptr as *const u8, size_of::<UDPHeader>()) + self.data.clone();
        }
    }
}

impl Response for UDP {
    #[allow(unused_variables)]
    fn respond(&self, session: &Session) -> Vector<Vector<u8>> {
        d("            ");
        self.d();
        dl();

        return Vector::new();
    }
}

impl UDP {
    pub fn d(&self){
        d("UDP from ");
        dd(self.header.src.get() as usize);
        d(" to ");
        dd(self.header.dst.get() as usize);
        d(" data ");
        dd(self.data.len());
    }
}
