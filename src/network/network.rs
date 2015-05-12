use common::debug::*;

#[derive(Copy, Clone)]
pub struct n16 {
    pub bytes: [u8; 2]
}

impl n16 {
    pub fn new(value: u16) -> n16{
        n16 {
            bytes: [
                (value >> 8) as u8,
                value as u8
            ]
        }
    }

    pub fn get(&self) -> u16 {
        return ((self.bytes[0] as u16) << 8) | (self.bytes[1] as u16);
    }

    pub fn set(&mut self, value: u16){
        self.bytes[0] = (value >> 8) as u8;
        self.bytes[1] = value as u8;
    }
}

#[derive(Copy, Clone)]
pub struct n32 {
    pub bytes: [u8; 4]
}

impl n32 {
    pub fn new(value: u32) -> n32{
        n32 {
            bytes: [
                (value >> 24) as u8,
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8
            ]
        }
    }

    pub fn get(&self) -> u32 {
        return ((self.bytes[0] as u32) << 24) | ((self.bytes[1] as u32) << 16) | ((self.bytes[2] as u32) << 8) | (self.bytes[3] as u32);
    }

    pub fn set(&mut self, value: u32){
        self.bytes[0] = (value >> 24) as u8;
        self.bytes[1] = (value >> 16) as u8;
        self.bytes[2] = (value >> 8) as u8;
        self.bytes[3] = value as u8;
    }
}

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

#[derive(Copy, Clone)]
pub struct EthernetII {
    pub dst: MACAddr,
    pub src: MACAddr,
    pub _type: n16
}

impl EthernetII {
    pub fn d(&self){
        d("Ethernet II ");
        dh(self._type.get() as usize);
        d(" from ");
        self.src.d();
        d(" to ");
        self.dst.d();
    }
}

#[derive(Copy, Clone)]
pub struct Checksum {
    pub data: u16
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

#[derive(Copy, Clone)]
pub struct IPv4 {
    pub ver_hlen: u8,
    pub services: u8,
    pub len: n16,
    pub id: n16,
    pub flags_fragment: n16,
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

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
pub struct TCP {
    pub src: n16,
    pub dst: n16,
    pub sequence: n32,
    pub ack_num: n32,
    pub flags: u16,
    pub window_size: n16,
    pub checksum: Checksum,
    pub urgent_pointer: n16
}

impl TCP {
    pub fn d(&self){
        d("TCP from ");
        dd(self.src.get() as usize);
        d(" to ");
        dd(self.dst.get() as usize);
    }
}

//Psuedo header for checksum only
pub struct TCPIPv4Psuedo {
    pub src_addr: IPv4Addr,
    pub dst_addr: IPv4Addr,
    pub zero: u8,
    pub proto: u8,
    pub tcp_len: n16,
    pub tcp: TCP
}

impl TCPIPv4Psuedo {
    pub fn new(packet: &IPv4, segment: &TCP) -> TCPIPv4Psuedo{
        TCPIPv4Psuedo {
            src_addr: packet.src,
            dst_addr: packet.dst,
            zero: 0,
            proto: packet.proto,
            tcp_len: n16::new(40),
            tcp: *segment
        }
    }
}

#[derive(Copy, Clone)]
pub struct UDP {
    pub src: n16,
    pub dst: n16,
    pub len: n16,
    pub checksum: Checksum
}

impl UDP {
    pub fn d(&self){
        d("UDP from ");
        dd(self.src.get() as usize);
        d(" to ");
        dd(self.dst.get() as usize);
    }
}

#[derive(Copy, Clone)]
pub struct ARP {
    pub htype: n16,
    pub ptype: n16,
    pub hlen: u8,
    pub plen: u8,
    pub oper: n16,
    pub src_mac: MACAddr,
    pub src_ip: IPv4Addr,
    pub dst_mac: MACAddr,
    pub dst_ip: IPv4Addr
}

impl ARP {
    pub fn d(&self){
        d("ARP hw ");
        dh(self.htype.get() as usize);
        d("#");
        dd(self.hlen as usize);
        d(" proto ");
        dh(self.ptype.get() as usize);
        d("#");
        dd(self.plen as usize);
        d(" oper ");
        dh(self.oper.get() as usize);
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

#[derive(Copy, Clone)]
pub struct ICMPv6 {
    pub _type: u8,
    pub code: u8,
    pub checksum: Checksum,
    pub body: n32
}
