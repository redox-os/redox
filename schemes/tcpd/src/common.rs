use std::{mem, slice, u8, u16, u32};

pub static mut IP_ADDR: Ipv4Addr = Ipv4Addr { bytes: [10, 0, 2, 15] };

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
#[allow(non_camel_case_types)]
#[repr(packed)]
pub struct n32(u32);

impl n32 {
    pub fn new(value: u32) -> Self {
        n32(value.to_be())
    }

    pub fn get(&self) -> u32 {
        u32::from_be(self.0)
    }

    pub fn set(&mut self, value: u32) {
        self.0 = value.to_be();
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

pub const TCP_FIN: u16 = 1;
pub const TCP_SYN: u16 = 1 << 1;
pub const TCP_RST: u16 = 1 << 2;
pub const TCP_PSH: u16 = 1 << 3;
pub const TCP_ACK: u16 = 1 << 4;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct TcpHeader {
    pub src: n16,
    pub dst: n16,
    pub sequence: n32,
    pub ack_num: n32,
    pub flags: n16,
    pub window_size: n16,
    pub checksum: Checksum,
    pub urgent_pointer: n16,
}

pub struct Tcp {
    pub header: TcpHeader,
    pub options: Vec<u8>,
    pub data: Vec<u8>,
}

impl Tcp {
    pub fn checksum(&mut self, src_addr: &Ipv4Addr, dst_addr: &Ipv4Addr) {
        self.header.checksum.data = 0;

        let proto = n16::new(0x06);
        let segment_len = n16::new((mem::size_of::<TcpHeader>() + self.options.len() + self.data.len()) as u16);
        self.header.checksum.data = Checksum::compile(unsafe {
            Checksum::sum(src_addr.bytes.as_ptr() as usize, src_addr.bytes.len()) +
            Checksum::sum(dst_addr.bytes.as_ptr() as usize, dst_addr.bytes.len()) +
            Checksum::sum((&proto as *const n16) as usize, mem::size_of::<n16>()) +
            Checksum::sum((&segment_len as *const n16) as usize, mem::size_of::<n16>()) +
            Checksum::sum((&self.header as *const TcpHeader) as usize, mem::size_of::<TcpHeader>()) +
            Checksum::sum(self.options.as_ptr() as usize, self.options.len()) +
            Checksum::sum(self.data.as_ptr() as usize, self.data.len())
        });
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() >= mem::size_of::<TcpHeader>() {
            unsafe {
                let header = *(bytes.as_ptr() as *const TcpHeader);
                let header_len = ((header.flags.get() & 0xF000) >> 10) as usize;

                return Some(Tcp {
                    header: header,
                    options: bytes[mem::size_of::<TcpHeader>()..header_len].to_vec(),
                    data: bytes[header_len..bytes.len()].to_vec(),
                });
            }
        }
        None
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let header_ptr: *const TcpHeader = &self.header;
            let mut ret = Vec::from(slice::from_raw_parts(header_ptr as *const u8,
                                                          mem::size_of::<TcpHeader>()));
            ret.extend_from_slice(&self.options);
            ret.extend_from_slice(&self.data);
            ret
        }
    }
}
