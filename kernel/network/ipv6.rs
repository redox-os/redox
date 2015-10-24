// TODO

use common::debug;

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

impl IPv6 {
    pub fn d(&self) {
        debug::d("IPv6 ");
        debug::dh(self.next_header as usize);
        debug::d(" from ");
        self.src.d();
        debug::d(" to ");
        self.dst.d();
    }
}
