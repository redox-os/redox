use netutils::{getcfg, n16, Ipv4Addr, MacAddr, Ipv4, EthernetII, EthernetIIHeader, Arp};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Result, Read, Write};
use std::os::unix::io::FromRawFd;

use interface::Interface;

pub struct EthernetInterface {
    mac: MacAddr,
    ip: Ipv4Addr,
    router: Ipv4Addr,
    subnet: Ipv4Addr,
    arp_file: File,
    ip_file: File,
    arp: BTreeMap<Ipv4Addr, MacAddr>,
    rarp: BTreeMap<MacAddr, Ipv4Addr>,
}

impl EthernetInterface {
    pub fn new(arp_fd: usize, ip_fd: usize) -> Self {
        EthernetInterface {
            mac: MacAddr::from_str(&getcfg("mac").unwrap()),
            ip: Ipv4Addr::from_str(&getcfg("ip").unwrap()),
            router: Ipv4Addr::from_str(&getcfg("ip_router").unwrap()),
            subnet: Ipv4Addr::from_str(&getcfg("ip_subnet").unwrap()),
            arp_file: unsafe { File::from_raw_fd(arp_fd) },
            ip_file: unsafe { File::from_raw_fd(ip_fd) },
            arp: BTreeMap::new(),
            rarp: BTreeMap::new(),
        }
    }
}

impl Interface for EthernetInterface {
    fn ip(&self) -> Ipv4Addr {
        self.ip
    }

    fn routable(&self, dst: Ipv4Addr) -> bool {
        dst != Ipv4Addr::LOOPBACK
    }

    fn arp_event(&mut self) -> Result<()> {
        loop {
            let mut bytes = [0; 65536];
            let count = self.arp_file.read(&mut bytes)?;
            if count == 0 {
                break;
            }
            if let Some(frame) = EthernetII::from_bytes(&bytes[.. count]) {
                if let Some(packet) = Arp::from_bytes(&frame.data) {
                    if packet.header.oper.get() == 1 {
                        if packet.header.dst_ip == self.ip {
                            if packet.header.src_ip != Ipv4Addr::BROADCAST && frame.header.src != MacAddr::BROADCAST {
                                self.arp.insert(packet.header.src_ip, frame.header.src);
                                self.rarp.insert(frame.header.src, packet.header.src_ip);
                            }

                            let mut response = Arp {
                                header: packet.header,
                                data: packet.data.clone(),
                            };
                            response.header.oper.set(2);
                            response.header.dst_mac = packet.header.src_mac;
                            response.header.dst_ip = packet.header.src_ip;
                            response.header.src_mac = self.mac;
                            response.header.src_ip = self.ip;

                            let mut response_frame = EthernetII {
                                header: frame.header,
                                data: response.to_bytes()
                            };

                            response_frame.header.dst = response_frame.header.src;
                            response_frame.header.src = self.mac;

                            self.arp_file.write(&response_frame.to_bytes())?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn recv(&mut self) -> Result<Vec<Ipv4>> {
        let mut ips = Vec::new();

        loop {
            let mut bytes = [0; 65536];
            let count = self.ip_file.read(&mut bytes)?;
            if count == 0 {
                break;
            }
            if let Some(frame) = EthernetII::from_bytes(&bytes[.. count]) {
                if let Some(ip) = Ipv4::from_bytes(&frame.data) {
                    if ip.header.dst == self.ip || ip.header.dst == Ipv4Addr::BROADCAST {
                        //TODO: Handle ping here

                        if ip.header.src != Ipv4Addr::BROADCAST && frame.header.src != MacAddr::BROADCAST {
                            self.arp.insert(ip.header.src, frame.header.src);
                            self.rarp.insert(frame.header.src, ip.header.src);
                        }

                        ips.push(ip);
                    }
                }
            }
        }

        Ok(ips)
    }

    fn send(&mut self, ip: Ipv4) -> Result<usize> {
        let mut dst = MacAddr::BROADCAST;
        if ip.header.dst != Ipv4Addr::BROADCAST {
            let mut needs_routing = false;

            for octet in 0..4 {
                let me = self.ip.bytes[octet];
                let mask = self.subnet.bytes[octet];
                let them = ip.header.dst.bytes[octet];
                if me & mask != them & mask {
                    needs_routing = true;
                    break;
                }
            }

            let route_addr = if needs_routing {
                self.router
            } else {
                ip.header.dst
            };

            if let Some(mac) = self.arp.get(&route_addr) {
                dst = *mac;
            } else {
                println!("ipd: need to arp {}", route_addr.to_string());
            }
        }

        let frame = EthernetII {
            header: EthernetIIHeader {
                dst: dst,
                src: self.mac,
                ethertype: n16::new(0x800),
            },
            data: ip.to_bytes()
        };

        self.ip_file.write(&frame.to_bytes())
    }
}
