// TODO

#[derive(Copy, Clone)]
pub struct IPv6 {
    pub version: n32, // also has traffic class and flow label, TODO
    pub len: n16,
    pub next_header: u8,
    pub hop_limit: u8,
    pub src: IPv6Addr,
    pub dst: IPv6Addr
}

impl IPv6 {
    pub fn d(&self){
        d("IPv6 ");
        dh(self.next_header as usize);
        d(" from ");
        self.src.d();
        d(" to ");
        self.dst.d();
    }
}