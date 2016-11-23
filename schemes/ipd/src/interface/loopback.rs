use netutils::{Ipv4Addr, Ipv4};
use std::io::Result;

use interface::Interface;

pub struct LoopbackInterface {
    ip: Ipv4Addr,
    packets: Vec<Ipv4>
}

impl LoopbackInterface {
    pub fn new() -> Self {
        LoopbackInterface {
            ip: Ipv4Addr::LOOPBACK,
            packets: Vec::new()
        }
    }
}

impl Interface for LoopbackInterface {
    fn ip(&self) -> Ipv4Addr {
        self.ip
    }

    fn recv(&mut self) -> Result<Vec<Ipv4>> {
        let mut ips = Vec::new();

        for ip in self.packets.drain(..) {
            ips.push(ip);
        }

        Ok(ips)
    }

    fn send(&mut self, ip: Ipv4) -> Result<usize> {
        self.packets.push(ip);

        Ok(0)
    }

    fn arp_event(&mut self) -> Result<()> {
        Ok(())
    }

    fn has_loopback_data(&self) -> bool {
        ! self.packets.is_empty()
    }
}
