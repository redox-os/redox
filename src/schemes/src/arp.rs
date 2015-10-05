use common::resource::URL;
use common::string::{String, ToString};
use common::vec::Vec;

use network::arp::*;
use network::common::*;

use programs::common::SessionItem;

use syscall::call;

pub struct ARPScheme;

impl SessionItem for ARPScheme {
    fn scheme(&self) -> String {
        return "arp".to_string();
    }
}

impl ARPScheme {
    pub fn reply_loop() {
        loop {
            let mut link = URL::from_str("ethernet:///806").open();

            let mut bytes: Vec<u8> = Vec::new();
            match link.read_to_end(&mut bytes) {
                Option::Some(_) => {
                    if let Option::Some(packet) = ARP::from_bytes(bytes) {
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

                            link.write(response.to_bytes().as_slice());
                        }
                    }
                }
                Option::None => call::sys_yield(),
            }
        }
    }
}
