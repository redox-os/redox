use common::debug;
use common::get_slice::GetSlice;

use collections::vec::Vec;

use core::{mem, slice};

use scheduler::context::context_switch;

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
                    data: bytes.get_slice(Some(mem::size_of::<ArpHeader>()), None).to_vec(),
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
            ret.push_all(&self.data);
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
        while let Ok(mut link) = Url::from_str("ethernet:/806").open() {
            loop {
                let mut bytes: Vec<u8> = Vec::new();
                if let Ok(_) = link.read_to_end(&mut bytes) {
                    if let Some(packet) = Arp::from_bytes(bytes) {
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

                            link.write(&response.to_bytes());
                        }
                    }
                } else {
                    break;
                }
            }
            unsafe { context_switch(false) };
        }
        debug::d("ARP: Failed to open ethernet:\n");
    }
}
