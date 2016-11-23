use netutils::{Ipv4, Ipv4Addr};
use std::io::Result;

pub use self::ethernet::EthernetInterface;
pub use self::loopback::LoopbackInterface;

mod ethernet;
mod loopback;

pub trait Interface {
    fn ip(&self) -> Ipv4Addr;
    fn recv(&mut self) -> Result<Vec<Ipv4>>;
    fn send(&mut self, ip: Ipv4) -> Result<usize>;

    fn arp_event(&mut self) -> Result<()>;

    fn has_loopback_data(&self) -> bool { false }
}
