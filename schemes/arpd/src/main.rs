extern crate syscall;

use std::thread;

use common::{MAC_ADDR, IP_ADDR, Arp};

pub mod common;

fn main() {
    thread::spawn(move || {
        while let Ok(link) = syscall::open("ethernet:/806", syscall::O_RDWR) {
            loop {
                let mut bytes = [0; 65536];
                if let Ok(count) = syscall::read(link, &mut bytes) {
                    if let Some(packet) = Arp::from_bytes(&bytes[..count]) {
                        if packet.header.oper.get() == 1 && packet.header.dst_ip.equals(unsafe { IP_ADDR }) {
                            let mut response = Arp {
                                header: packet.header,
                                data: packet.data.clone(),
                            };
                            response.header.oper.set(2);
                            response.header.dst_mac = packet.header.src_mac;
                            response.header.dst_ip = packet.header.src_ip;
                            response.header.src_mac = unsafe { MAC_ADDR };
                            response.header.src_ip = unsafe { IP_ADDR };

                            let _ = syscall::write(link, &response.to_bytes());
                        }
                    }
                } else {
                    break;
                }
            }
            let _ = syscall::close(link);
        }
        panic!("ARP: Failed to open ethernet");
    });
}
