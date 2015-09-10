use core::option::Option;

use common::debug::*;
use common::string::*;
use common::vec::*;

pub trait FromBytes{
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> where Self:Sized;
}

pub trait ToBytes{
    fn to_bytes(&self) -> Vec<u8>;
}

#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
pub struct n16 {
    pub bytes: [u8; 2]
}

impl n16 {
    pub fn new(value: u16) -> n16{
        n16 {
            bytes: [
                (value >> 8) as u8,
                value as u8
            ]
        }
    }

    pub fn get(&self) -> u16 {
        return ((self.bytes[0] as u16) << 8) | (self.bytes[1] as u16);
    }

    pub fn set(&mut self, value: u16){
        self.bytes[0] = (value >> 8) as u8;
        self.bytes[1] = value as u8;
    }
}

#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
pub struct n32 {
    pub bytes: [u8; 4]
}

impl n32 {
    pub fn new(value: u32) -> n32{
        n32 {
            bytes: [
                (value >> 24) as u8,
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8
            ]
        }
    }

    pub fn get(&self) -> u32 {
        return ((self.bytes[0] as u32) << 24) | ((self.bytes[1] as u32) << 16) | ((self.bytes[2] as u32) << 8) | (self.bytes[3] as u32);
    }

    pub fn set(&mut self, value: u32){
        self.bytes[0] = (value >> 24) as u8;
        self.bytes[1] = (value >> 16) as u8;
        self.bytes[2] = (value >> 8) as u8;
        self.bytes[3] = value as u8;
    }
}

#[derive(Copy, Clone)]
pub struct MACAddr {
    pub bytes: [u8; 6]
}

impl MACAddr {
    pub fn equals(&self, other: MACAddr) -> bool {
        for i in 0..6 {
            if self.bytes[i] != other.bytes[i] {
                return false;
            }
        }
        return true;
    }

    pub fn from_string(string: &String) -> MACAddr {
        let mut addr = MACAddr {
            bytes: [0, 0, 0, 0, 0, 0]
        };

        let mut i = 0;
        for part in string.split(".".to_string()) {
            let octet = part.to_num_radix(16) as u8;
            match i {
                0 => addr.bytes[0] = octet,
                1 => addr.bytes[1] = octet,
                2 => addr.bytes[2] = octet,
                3 => addr.bytes[3] = octet,
                4 => addr.bytes[4] = octet,
                5 => addr.bytes[5] = octet,
                _ => break
            }
            i += 1;
        }

        return addr;
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();
        for i in 0..6 {
            if i > 0 {
                string = string + '.';
            }
            string = string + String::from_num_radix(self.bytes[i] as usize, 16);
        }
        return string;
    }

    pub fn d(&self){
        self.to_string().d();
    }
}

pub static BROADCAST_MAC_ADDR: MACAddr = MACAddr {
    bytes: [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
};

pub static MAC_ADDR: MACAddr = MACAddr {
    bytes: [0x52, 0x54, 0x00, 0x12, 0x34, 0x56]
};

#[derive(Copy, Clone)]
pub struct IPv4Addr {
    pub bytes: [u8; 4]
}

impl IPv4Addr {
    pub fn equals(&self, other: IPv4Addr) -> bool {
        for i in 0..4 {
            if self.bytes[i] != other.bytes[i] {
                return false;
            }
        }
        return true;
    }

    pub fn from_string(string: &String) -> IPv4Addr {
        let mut addr = IPv4Addr {
            bytes: [0, 0, 0, 0]
        };

        let mut i = 0;
        for part in string.split(".".to_string()) {
            let octet = part.to_num() as u8;
            match i {
                0 => addr.bytes[0] = octet,
                1 => addr.bytes[1] = octet,
                2 => addr.bytes[2] = octet,
                3 => addr.bytes[3] = octet,
                _ => break
            }
            i += 1;
        }

        return addr;
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();

        for i in 0..4 {
            if i > 0 {
                string = string + '.';
            }
            string = string + self.bytes[i] as usize;
        }

        return string;
    }

    pub fn d(&self){
        self.to_string().d();
    }
}

#[derive(Copy, Clone)]
pub struct IPv6Addr {
    pub bytes: [u8; 16]
}

impl IPv6Addr {
    pub fn d(&self){
        for i in 0..16 {
            if i > 0 && i % 2 == 0 {
                d(":");
            }
            dbh(self.bytes[i]);
        }
    }
}

pub static BROADCAST_IP_ADDR: IPv4Addr = IPv4Addr {
    bytes: [10, 85, 85, 255]
};

pub static IP_ADDR: IPv4Addr = IPv4Addr {
    bytes: [10, 85, 85, 2]
};

#[derive(Copy, Clone)]
pub struct Checksum {
    pub data: u16
}

impl Checksum {
    pub unsafe fn check(&self, mut ptr: usize, mut len: usize) -> bool{
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

        return sum == 0xFFFF;
    }

    pub unsafe fn calculate(&mut self, ptr: usize, len: usize){
        self.data = 0;

        let sum = Checksum::sum(ptr, len);

        self.data = Checksum::compile(sum);
    }

    pub unsafe fn sum(mut ptr: usize, mut len: usize) -> usize{
        let mut sum = 0;

        while len > 1 {
            sum += *(ptr as *const u16) as usize;
            len -= 2;
            ptr += 2;
        }

        if len > 0 {
            sum += *(ptr as *const u8) as usize;
        }

        return sum;
    }

    pub unsafe fn compile(mut sum: usize) -> u16{
        while (sum >> 16) > 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        return 0xFFFF - (sum as u16);
    }
}
