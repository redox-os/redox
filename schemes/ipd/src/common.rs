use std::{mem, slice, u8, u16};

pub static mut MAC_ADDR: MacAddr = MacAddr { bytes: [0x50, 0x51, 0x52, 0x53, 0x54, 0x55] };
pub static BROADCAST_MAC_ADDR: MacAddr = MacAddr { bytes: [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] };

pub static mut IP_ADDR: Ipv4Addr = Ipv4Addr { bytes: [10, 0, 2, 15] };
pub static mut IP_ROUTER_ADDR: Ipv4Addr = Ipv4Addr { bytes: [10, 0, 2, 2] };
pub static mut IP_SUBNET: Ipv4Addr = Ipv4Addr { bytes: [255, 255, 255, 0] };
pub static BROADCAST_IP_ADDR: Ipv4Addr = Ipv4Addr { bytes: [255, 255, 255, 255] };

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
pub struct Ipv4Addr {
    pub bytes: [u8; 4],
}

impl Ipv4Addr {
    pub fn equals(&self, other: Self) -> bool {
        for i in 0..4 {
            if self.bytes[i] != other.bytes[i] {
                return false;
            }
        }
        true
    }

    pub fn from_str(string: &str) -> Self {
        let mut addr = Ipv4Addr { bytes: [0, 0, 0, 0] };

        let mut i = 0;
        for part in string.split('.') {
            let octet = part.parse::<u8>().unwrap_or(0);
            match i {
                0 => addr.bytes[0] = octet,
                1 => addr.bytes[1] = octet,
                2 => addr.bytes[2] = octet,
                3 => addr.bytes[3] = octet,
                _ => break,
            }
            i += 1;
        }

        addr
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();

        for i in 0..4 {
            if i > 0 {
                string = string + ".";
            }
            string = string + &format!("{}", self.bytes[i]);
        }

        string
    }
}

#[derive(Copy, Clone)]
pub struct Checksum {
    pub data: u16,
}

impl Checksum {
    pub unsafe fn check(&self, mut ptr: usize, mut len: usize) -> bool {
        let mut sum: usize = 0;
        while len > 1 {
            sum += *(ptr as *const u16) as usize;
            len -= 2;
            ptr += 2;
        }

        if len > 0 {
            sum += *(ptr as *const u8) as usize;
        }

        while (sum >> 16) > 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        sum == 0xFFFF
    }

    pub unsafe fn calculate(&mut self, ptr: usize, len: usize) {
        self.data = 0;

        let sum = Checksum::sum(ptr, len);

        self.data = Checksum::compile(sum);
    }

    pub unsafe fn sum(mut ptr: usize, mut len: usize) -> usize {
        let mut sum = 0;

        while len > 1 {
            sum += *(ptr as *const u16) as usize;
            len -= 2;
            ptr += 2;
        }

        if len > 0 {
            sum += *(ptr as *const u8) as usize;
        }

        sum
    }

    pub fn compile(mut sum: usize) -> u16 {
        while (sum >> 16) > 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        0xFFFF - (sum as u16)
    }
}

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct ArpHeader {
    pub htype: n16,
    pub ptype: n16,
    pub hlen: u8,
    pub plen: u8,
    pub oper: n16,
    pub src_mac: MacAddr,
    pub src_ip: Ipv4Addr,
    pub dst_mac: MacAddr,
    pub dst_ip: Ipv4Addr,
}

pub struct Arp {
    pub header: ArpHeader,
    pub data: Vec<u8>,
}

impl Arp {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() >= mem::size_of::<ArpHeader>() {
            unsafe {
                return Some(Arp {
                    header: *(bytes.as_ptr() as *const ArpHeader),
                    data: bytes[mem::size_of::<ArpHeader>() ..].to_vec(),
                });
            }
        }
        None
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let header_ptr: *const ArpHeader = &self.header;
            let mut ret = Vec::from(slice::from_raw_parts(header_ptr as *const u8,
                                                          mem::size_of::<ArpHeader>()));
            ret.extend_from_slice(&self.data);
            ret
        }
    }
}

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Ipv4Header {
    pub ver_hlen: u8,
    pub services: u8,
    pub len: n16,
    pub id: n16,
    pub flags_fragment: n16,
    pub ttl: u8,
    pub proto: u8,
    pub checksum: Checksum,
    pub src: Ipv4Addr,
    pub dst: Ipv4Addr,
}

pub struct Ipv4 {
    pub header: Ipv4Header,
    pub options: Vec<u8>,
    pub data: Vec<u8>,
}

impl Ipv4 {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() >= mem::size_of::<Ipv4Header>() {
            unsafe {
                let header = *(bytes.as_ptr() as *const Ipv4Header);
                let header_len = ((header.ver_hlen & 0xF) << 2) as usize;

                return Some(Ipv4 {
                    header: header,
                    options: bytes[mem::size_of::<Ipv4Header>() .. header_len].to_vec(),
                    data: bytes[header_len .. header.len.get() as usize].to_vec(),
                });
            }
        }
        None
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let header_ptr: *const Ipv4Header = &self.header;
            let mut ret = Vec::<u8>::from(slice::from_raw_parts(header_ptr as *const u8,
                                                                mem::size_of::<Ipv4Header>()));
            ret.extend_from_slice(&self.options);
            ret.extend_from_slice(&self.data);
            ret
        }
    }
}
