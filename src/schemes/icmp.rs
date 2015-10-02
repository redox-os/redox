use network::common::*;
use network::icmp::*;

use programs::common::*;

pub struct ICMPScheme;

impl SessionItem for ICMPScheme {
    fn scheme(&self) -> String {
        return "icmp".to_string();
    }
}

impl ICMPScheme {
    pub fn reply_loop() {
        loop {
            let mut ip = URL::from_str("ip:///1").open();

            let mut bytes: Vec<u8> = Vec::new();
            match ip.read_to_end(&mut bytes) {
                Option::Some(_) => {
                    if let Option::Some(message) = ICMP::from_bytes(bytes) {
                        if message.header._type == 0x08 {
                            let mut response = ICMP {
                                header: message.header,
                                data: message.data,
                            };

                            response.header._type = 0x00;

                            unsafe {
                                response.header.checksum.data = 0;

                                let header_ptr: *const ICMPHeader = &response.header;
                                response.header.checksum.data = Checksum::compile(
                                    Checksum::sum(header_ptr as usize, size_of::<ICMPHeader>()) +
                                    Checksum::sum(response.data.as_ptr() as usize, response.data.len())
                                );
                            }

                            ip.write(&response.to_bytes().as_slice());
                        }
                    }
                }
                Option::None => sys_yield(),
            }
        }
    }
}
