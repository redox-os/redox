use collections::vec::Vec;

use core::{mem, slice};

use scheduler::context::recursive_unsafe_yield;

use network::common::*;

use schemes::{KScheme, URL};

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct ARPHeader {
    pub htype: n16,
    pub ptype: n16,
    pub hlen: u8,
    pub plen: u8,
    pub oper: n16,
    pub src_mac: MACAddr,
    pub src_ip: IPv4Addr,
    pub dst_mac: MACAddr,
    pub dst_ip: IPv4Addr,
}

pub struct ARP {
    pub header: ARPHeader,
    pub data: Vec<u8>,
}

impl FromBytes for ARP {
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() >= mem::size_of::<ARPHeader>() {
            unsafe {
                return Some(ARP {
                    header: *(bytes.as_ptr() as *const ARPHeader),
                    data: bytes[mem::size_of::<ARPHeader>() ..].to_vec()
                });
            }
        }
        None
    }
}

impl ToBytes for ARP {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let header_ptr: *const ARPHeader = &self.header;
            let mut ret = Vec::from(slice::from_raw_parts(header_ptr as *const u8, mem::size_of::<ARPHeader>()));
            ret.push_all(&self.data);
            ret
        }
    }
}

pub struct ARPScheme;

impl KScheme for ARPScheme {
    fn scheme(&self) -> &str {
        "arp"
    }
}

impl ARPScheme {
    pub fn reply_loop() {
        while let Some(mut link) = URL::from_str("ethernet:///806").open() {
            loop {
                let mut bytes: Vec<u8> = Vec::new();
                if let Some(_) = link.read_to_end(&mut bytes) {
                    if let Some(packet) = ARP::from_bytes(bytes) {
                        if packet.header.oper.get() == 1 && packet.header.dst_ip.equals(IP_ADDR) {
                            let mut response = ARP {
                                header: packet.header,
                                data: packet.data.clone(),
                            };
                            response.header.oper.set(2);
                            response.header.dst_mac = packet.header.src_mac;
                            response.header.dst_ip = packet.header.src_ip;
                            response.header.src_mac = unsafe { MAC_ADDR };
                            response.header.src_ip = IP_ADDR;

                            link.write(&response.to_bytes());
                        }
                    }
                }else{
                    break;
                }
            }
            unsafe { recursive_unsafe_yield() }
        }
    }
}
