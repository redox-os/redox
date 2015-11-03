// TODO

use network::common::*;

#[derive(Copy, Clone)]
pub struct Ipv6 {
    pub version: n32, // also has traffic class and flow label, TODO
    pub len: n16,
    pub next_header: u8,
    pub hop_limit: u8,
    pub src: Ipv6Addr,
    pub dst: Ipv6Addr,
}
