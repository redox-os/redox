extern crate netutils;
extern crate syscall;

use netutils::{getcfg, Ipv4Addr, MacAddr, Arp};

use std::thread;

fn main() {
    thread::spawn(move || {
        while let Ok(link) = syscall::open("ethernet:/806", syscall::O_RDWR) {
            loop {
                let mut bytes = [0; 65536];
                if let Ok(count) = syscall::read(link, &mut bytes) {
                    if let Some(packet) = Arp::from_bytes(&bytes[..count]) {
                        let mac_addr = MacAddr::from_str(&getcfg("mac").expect("arpd: failed to get mac address"));
                        let ip_addr = Ipv4Addr::from_str(&getcfg("ip").expect("arpd: failed to get ip address"));

                        if packet.header.oper.get() == 1 && packet.header.dst_ip.equals(ip_addr) {
                            let mut response = Arp {
                                header: packet.header,
                                data: packet.data.clone(),
                            };
                            response.header.oper.set(2);
                            response.header.dst_mac = packet.header.src_mac;
                            response.header.dst_ip = packet.header.src_ip;
                            response.header.src_mac = mac_addr;
                            response.header.src_ip = ip_addr;

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
