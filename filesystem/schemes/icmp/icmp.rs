use redox::Box;
use redox::cell::UnsafeCell;
use redox::console::ConsoleWindow;
use redox::fs::file::File;
use redox::rc::Rc;
use redox::str;
use redox::string::*;
use redox::io::SeekFrom;

pub struct Scheme;

impl Scheme {
    pub fn scheme(&self) -> Box<Self> {
        box Scheme
    }

    pub fn reply_loop() {
        while let Some(mut ip) = File::open("ip:///1") {
            let mut bytes: Vec<u8> = Vec::new();
            match ip.read_to_end(&mut bytes) {
                Some(_) => {
                    if let Some(message) = ICMP::from_bytes(bytes) {
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
                                    Checksum::sum(header_ptr as usize, mem::size_of::<ICMPHeader>()) +
                                    Checksum::sum(response.data.as_ptr() as usize, response.data.len())
                                );
                            }

                            ip.write(&response.to_bytes().as_slice());
                        }
                    }
                }
                None => unsafe { context_switch(false) },
            }
        }
    }

    /*
    pub fn open(&mut self, url: &str) -> Option<Box<Resource>> {
    }
    */
}
