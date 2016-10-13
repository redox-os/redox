use std::{mem, slice, u8, u16};

#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
#[repr(packed)]
pub struct n16(u16);

impl n16 {
    pub fn new(value: u16) -> Self {
        n16(value.to_be())
    }

    pub fn get(&self) -> u16 {
        u16::from_be(self.0)
    }

    pub fn set(&mut self, value: u16) {
        self.0 = value.to_be();
    }
}

#[derive(Copy, Clone)]
pub struct MacAddr {
    pub bytes: [u8; 6],
}

impl MacAddr {
    pub fn equals(&self, other: Self) -> bool {
        for i in 0..6 {
            if self.bytes[i] != other.bytes[i] {
                return false;
            }
        }
        true
    }

    pub fn from_str(string: &str) -> Self {
        let mut addr = MacAddr { bytes: [0, 0, 0, 0, 0, 0] };

        let mut i = 0;
        for part in string.split('.') {
            let octet = u8::from_str_radix(part, 16).unwrap_or(0);
            match i {
                0 => addr.bytes[0] = octet,
                1 => addr.bytes[1] = octet,
                2 => addr.bytes[2] = octet,
                3 => addr.bytes[3] = octet,
                4 => addr.bytes[4] = octet,
                5 => addr.bytes[5] = octet,
                _ => break,
            }
            i += 1;
        }

        addr
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();
        for i in 0..6 {
            if i > 0 {
                string.push('.');
            }
            string.push_str(&format!("{:X}", self.bytes[i]));
        }
        string
    }
}

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct EthernetIIHeader {
    pub dst: MacAddr,
    pub src: MacAddr,
    pub ethertype: n16,
}

pub struct EthernetII {
    pub header: EthernetIIHeader,
    pub data: Vec<u8>,
}

impl EthernetII {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() >= mem::size_of::<EthernetIIHeader>() {
            unsafe {
                return Some(EthernetII {
                    header: *(bytes.as_ptr() as *const EthernetIIHeader),
                    data: bytes[mem::size_of::<EthernetIIHeader>() ..].to_vec(),
                });
            }
        }
        None
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let header_ptr: *const EthernetIIHeader = &self.header;
            let mut ret = Vec::from(slice::from_raw_parts(header_ptr as *const u8,
                                                          mem::size_of::<EthernetIIHeader>()));
            ret.extend_from_slice(&self.data);
            ret
        }
    }
}
