use core::mem::size_of;

use common::debug::*;

pub trait NetworkDevice {
    unsafe fn send(&self, ptr: usize, len: usize);
}

#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
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
#[allow(non_camel_case_types)]
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
    pub urgent_pointer: n16,
    pub options: [u8; 20]
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
    pub segment_len: n16,
    pub segment: TCP
}

impl TCPIPv4Psuedo {
    pub fn new(packet: &IPv4, segment: &TCP) -> TCPIPv4Psuedo{
        TCPIPv4Psuedo {
            src_addr: packet.src,
            dst_addr: packet.dst,
            zero: 0,
            proto: packet.proto,
            segment_len: n16::new(packet.len.get() - ((packet.ver_hlen & 0xF) as u16)*4),
            segment: *segment
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

unsafe fn network_icmpv4(device: &NetworkDevice, frame: &mut EthernetII, packet: &mut IPv4, segment_addr: usize){
    let segment = &mut *(segment_addr as *mut ICMP);
    d("        ");
    segment.d();
    dl();

    if segment._type == 0x08 && packet.dst.equals(IP_ADDR) {
        d("            Echo Reply\n");
        //Send echo reply
        frame.dst = frame.src;
        frame.src = MAC_ADDR;
        packet.dst = packet.src;
        packet.src = IP_ADDR;
        let id = packet.id.get();
        packet.id.set(id + 1);
        segment._type = 0x00;

        segment.checksum.calculate(segment_addr, packet.len.get() as usize - ((packet.ver_hlen & 0xF) as usize) * 4);

        let packet_addr: *const IPv4 = packet;
        packet.checksum.calculate(packet_addr as usize, ((packet.ver_hlen & 0xF) as usize) * 4);

        let frame_addr: *const EthernetII = frame;
        device.send(frame_addr as usize, size_of::<EthernetII>() + packet.len.get() as usize);
    }else{
        d("            Ignore ICMP\n");
    }
}

unsafe fn network_tcpv4(device: &NetworkDevice, frame: &mut EthernetII, packet: &mut IPv4, segment_addr: usize){
    let mut segment = &mut *(segment_addr as *mut TCP);
    d("        ");
    segment.d();
    dl();

    if segment.dst.get() == 80 {
        d("            HTTP Reply ");
        dh(segment.flags as usize);
        d(" ");
        dh(segment.sequence.get() as usize);
        d(" ");
        dh(segment.ack_num.get() as usize);
        dl();

        frame.dst = frame.src;
        frame.src = MAC_ADDR;
        packet.dst = packet.src;
        packet.src = IP_ADDR;
        segment.dst.set(segment.src.get());
        segment.src.set(80);

        let frame_addr: *const EthernetII = frame;
        let packet_addr: *const IPv4 = packet;

        if segment.flags & (1 << 12) == 0 {
            d("            HTTP SYN\n");
            segment.flags = segment.flags | (1 << 12);
            segment.ack_num.set(segment.sequence.get() + 1);
            segment.sequence.set(0x76543210); // TODO: Randomize

            segment.checksum.data = 0;

            let tcpip_psuedo = TCPIPv4Psuedo::new(packet, segment);
            let tcpip_psuedo_addr: *const TCPIPv4Psuedo = &tcpip_psuedo;
            segment.checksum.calculate(tcpip_psuedo_addr as usize, 52);

            packet.checksum.calculate(packet_addr as usize, ((packet.ver_hlen & 0xF) as usize)*4);

            device.send(frame_addr as usize, 74);
        }else if segment.flags & (1 << 11) == 0{
            d("            HTTP ACK\n");
        }else{
            d("            HTTP PSH\n");

            segment.flags = segment.flags & (0xFFFF - (1 << 11));
            segment.ack_num.set(segment.sequence.get() + 137);
            segment.sequence.set(0x65432109);

            packet.len.set(52);

            segment.checksum.data = 0;

            let tcpip_psuedo = TCPIPv4Psuedo::new(packet, segment);
            let tcpip_psuedo_addr: *const TCPIPv4Psuedo = &tcpip_psuedo;
            segment.checksum.calculate(tcpip_psuedo_addr as usize, 44);

            packet.checksum.calculate(packet_addr as usize, ((packet.ver_hlen & 0xF) as usize)*4);

            device.send(frame_addr as usize, 66);

            segment.flags = segment.flags | (1 << 11);
            let sequence = segment.sequence.get();
            segment.sequence.set(sequence);

            segment.checksum.data = 0;

            let tcpip_psuedo = TCPIPv4Psuedo::new(packet, segment);
            let tcpip_psuedo_addr: *const TCPIPv4Psuedo = &tcpip_psuedo;
            segment.checksum.calculate(tcpip_psuedo_addr as usize, 44);

            packet.checksum.calculate(packet_addr as usize, ((packet.ver_hlen & 0xF) as usize)*4);

            device.send(frame_addr as usize, 66);
        }
    }
}

unsafe fn network_udpv4(device: &NetworkDevice, frame: &mut EthernetII, packet: &mut IPv4, segment_addr: usize){
    let segment = &*(segment_addr as *const UDP);
    d("        ");
    segment.d();
    dl();
}

unsafe fn network_ipv4(device: &NetworkDevice, frame: &mut EthernetII, packet_addr: usize){
    let packet = &mut *(packet_addr as *mut IPv4);
    d("    ");
    packet.d();
    dl();

    let segment_addr = packet_addr + ((packet.ver_hlen & 0xF) as usize) * 4;

    if packet.proto == 0x01 {
        network_icmpv4(device, frame, packet, segment_addr);
    }else if packet.proto == 0x06 {
        network_tcpv4(device, frame, packet, segment_addr);
    }else if packet.proto == 0x11 {
        network_udpv4(device, frame, packet, segment_addr);
    }
}

unsafe fn network_udpv6(device: &NetworkDevice, frame: &mut EthernetII, packet: &mut IPv6, segment_addr: usize){
    let segment = &mut *(segment_addr as *mut UDP);
    d("        ");
    segment.d();
    dl();
}

unsafe fn network_ipv6(device: &NetworkDevice, frame: &mut EthernetII, packet_addr: usize){
    let packet = &mut *(packet_addr as *mut IPv6);
    d("    ");
    packet.d();
    dl();

    if packet.next_header == 0x11 {
        network_udpv6(device, frame, packet, packet_addr + size_of::<IPv6>())
    }
}

unsafe fn network_arp(device: &NetworkDevice, frame: &mut EthernetII, packet_addr: usize){
    let packet = &mut *(packet_addr as *mut ARP);
    d("    ");
    packet.d();
    dl();

    if packet.oper.get() == 1 && packet.dst_ip.equals(IP_ADDR) {
        d("        ARP Reply\n");
        //Send arp reply
        frame.dst = frame.src;
        frame.src = MAC_ADDR;
        packet.oper.set(2);
        packet.dst_mac = packet.src_mac;
        packet.dst_ip = packet.src_ip;
        packet.src_mac = MAC_ADDR;
        packet.src_ip = IP_ADDR;

        let frame_addr: *const EthernetII = frame;
        device.send(frame_addr as usize, size_of::<EthernetII>() + size_of::<ARP>());
    }else{
        d("        Ignore ARP\n");
    }
}

pub unsafe fn network_frame(device: &NetworkDevice, frame_addr: usize, frame_len: usize){
    let frame = &mut *(frame_addr as *mut EthernetII);
    frame.d();
    dl();

    if frame._type.get() == 0x0800 {
        network_ipv4(device, frame, frame_addr + size_of::<EthernetII>());
    }else if frame._type.get() == 0x0806 {
        network_arp(device, frame, frame_addr + size_of::<EthernetII>());
    }else if frame._type.get() == 0x86DD {
        network_ipv6(device, frame, frame_addr + size_of::<EthernetII>());
    }else{
        for ptr in frame_addr..frame_addr + frame_len {
            let data = *(ptr as *const u8);
            dbh(data);
            if (ptr - frame_addr) % 40 == 39 {
                dl();
            }else if (ptr - frame_addr) % 4 == 3{
                d(" ");
            }
        }
        dl();
    }
}
