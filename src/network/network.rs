use common::debug::*;

#[derive(Copy, Clone)]
pub struct MACAddr {
    pub bytes: [u8; 6]
}

impl MACAddr {
    pub fn d(&self){
        for i in 0..6 {
            if i > 0 {
                d(":");
            }
            dbh(self.bytes[i]);
        }
    }
}

pub static MAC_ADDR: MACAddr = MACAddr {
    bytes: [0x52, 0x54, 0x00, 0x12, 0x34, 0x56]
};

#[derive(Copy, Clone)]
pub struct IPv4Addr {
    pub bytes: [u8; 4]
}

impl IPv4Addr {
    pub fn equals(&self, other: IPv4Addr) -> bool {
        for i in 0..4 {
            if self.bytes[i] != other.bytes[i] {
                return false;
            }
        }
        return true;
    }

    pub fn d(&self){
        for i in 0..4 {
            if i > 0 {
                d(".");
            }
            dd(self.bytes[i] as usize);
        }
    }
}

pub static IP_ADDR: IPv4Addr = IPv4Addr {
    bytes: [10, 85, 85, 2]
};

#[derive(Copy, Clone)]
pub struct IPv6Addr {
    pub bytes: [u8; 16]
}

impl IPv6Addr {
    pub fn d(&self){
        for i in 0..16 {
            if i > 0 && i % 2 == 0 {
                d(":");
            }
            dbh(self.bytes[i]);
        }
    }
}

pub struct EthernetII {
    pub dst: MACAddr,
    pub src: MACAddr,
    pub _type: u16
}

impl EthernetII {
    pub fn d(&self){
        d("Ethernet II ");
        dh(self._type as usize);
        d(" from ");
        self.src.d();
        d(" to ");
        self.dst.d();
    }
}

pub struct Checksum {
    data: u16
}

impl Checksum {
    pub unsafe fn check(&self, mut ptr: usize, mut len: usize) -> bool{
        let mut sum: usize = 0;
        while len > 1 {
            sum += *(ptr as *const u16) as usize;
            len -= 2;
            ptr += 2;
        }

        if len > 0 {
            sum += *(ptr as *const u8) as usize;
        }

        while (sum >> 16) > 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        return sum == 0xFFFF;
    }

    pub unsafe fn calculate(&mut self, mut ptr: usize, mut len: usize){
        self.data = 0;

        let mut sum: usize = 0;
        while len > 1 {
            sum += *(ptr as *const u16) as usize;
            len -= 2;
            ptr += 2;
        }

        if len > 0 {
            sum += *(ptr as *const u8) as usize;
        }

        while (sum >> 16) > 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        self.data = 0xFFFF - (sum as u16);
    }
}

pub struct IPv4 {
    pub ver_hlen: u8,
    pub services: u8,
    pub len: u16,
    pub id: u16,
    pub flags_fragment: u16,
    pub ttl: u8,
    pub proto: u8,
    pub checksum: Checksum,
    pub src: IPv4Addr,
    pub dst: IPv4Addr
}

impl IPv4 {
    pub fn d(&self){
        d("IPv4 ");
        dbh(self.proto);
        d(" from ");
        self.src.d();
        d(" to ");
        self.dst.d();
    }
}

pub struct ICMP {
    pub _type: u8,
    pub code: u8,
    pub checksum: Checksum,
    pub data: [u8; 4]
}

impl ICMP {
    pub fn d(&self){
        d("ICMP ");
        dbh(self._type);
        d(" code ");
        dbh(self.code);
    }
}

pub struct TCP {
    pub src: [u8; 2],
    pub dst: [u8; 2],
    pub sequence: u32,
    pub ack_num: u32,
    pub flags: u16,
    pub window_size: u16,
    pub checksum: Checksum,
    pub urgent_pointer: u16
}

impl TCP {
    pub fn d(&self){
        d("TCP from ");
        dd(self.src[0] as usize * 256 + self.src[1] as usize);
        d(" to ");
        dd(self.dst[0] as usize * 256 + self.dst[1] as usize);
    }
}

pub struct UDP {
    pub src: [u8; 2],
    pub dst: [u8; 2],
    pub len: u16,
    pub checksum: Checksum
}

impl UDP {
    pub fn d(&self){
        d("UDP from ");
        dd(self.src[0] as usize * 256 + self.src[1] as usize);
        d(" to ");
        dd(self.dst[0] as usize * 256 + self.dst[1] as usize);
    }
}

pub struct ARP {
    pub htype: u16,
    pub ptype: u16,
    pub hlen: u8,
    pub plen: u8,
    pub oper: u16,
    pub src_mac: MACAddr,
    pub src_ip: IPv4Addr,
    pub dst_mac: MACAddr,
    pub dst_ip: IPv4Addr
}

impl ARP {
    pub fn d(&self){
        d("ARP hw ");
        dh(self.htype as usize);
        d("#");
        dd(self.hlen as usize);
        d(" proto ");
        dh(self.ptype as usize);
        d("#");
        dd(self.plen as usize);
        d(" oper ");
        dh(self.oper as usize);
        d(" from ");
        self.src_mac.d();
        d(" (");
        self.src_ip.d();
        d(") to ");
        self.dst_mac.d();
        d(" (");
        self.dst_ip.d();
        d(")");
    }
}

pub struct IPv6 {
    pub version: u32, // also has traffic class and flow label, TODO
    pub len: u16,
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

pub struct ICMPv6 {
    pub _type: u8,
    pub code: u8,
    pub checksum: Checksum,
    pub body: u32
}
