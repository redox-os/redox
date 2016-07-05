use common::slice::GetSlice;

use collections::vec::Vec;

use core::{mem, slice};

use arch::context::context_switch;

use network::common::*;

use fs::{KScheme, Url};

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

impl FromBytes for Arp {
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() >= mem::size_of::<ArpHeader>() {
            unsafe {
                return Some(Arp {
                    header: *(bytes.as_ptr() as *const ArpHeader),
                    data: bytes.get_slice(mem::size_of::<ArpHeader>() ..).to_vec(),
                });
            }
        }
        None
    }
}

impl ToBytes for Arp {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let header_ptr: *const ArpHeader = &self.header;
            let mut ret = Vec::from(slice::from_raw_parts(header_ptr as *const u8,
                                                          mem::size_of::<ArpHeader>()));
            ret.extend_from_slice(&self.data);
            ret
        }
    }
}

pub struct ArpScheme;

impl KScheme for ArpScheme {
    fn scheme(&self) -> &str {
        "arp"
    }
}

impl ArpScheme {
    pub fn reply_loop() {
        while let Ok(mut link) = Url::from_str("ethernet:/806").unwrap().open() {
            loop {
                let mut bytes = [0; 8192];
                if let Ok(count) = link.read(&mut bytes) {
                    if let Some(packet) = Arp::from_bytes(bytes[.. count].to_vec()) {
                        if packet.header.oper.get() == 1 && packet.header.dst_ip.equals(IP_ADDR) {
                            let mut response = Arp {
                                header: packet.header,
                                data: packet.data.clone(),
                            };
                            response.header.oper.set(2);
                            response.header.dst_mac = packet.header.src_mac;
                            response.header.dst_ip = packet.header.src_ip;
                            response.header.src_mac = unsafe { MAC_ADDR };
                            response.header.src_ip = IP_ADDR;

                            let _ = link.write(&response.to_bytes());
                        }
                    }
                } else {
                    break;
                }
            }
            unsafe { context_switch() };
        }
        debug!("ARP: Failed to open ethernet:\n");
    }
}
