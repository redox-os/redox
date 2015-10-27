// TODO

use network::common::*;

#[derive(Copy, Clone)]
pub struct IPv6 {
    pub version: n32, // also has traffic class and flow label, TODO
    pub len: n16,
    pub next_header: u8,
    pub hop_limit: u8,
    pub src: IPv6Addr,
    pub dst: IPv6Addr,
}
