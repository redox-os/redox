use core::mem::size_of;
use core::option::Option;
use core::slice;

use common::debug::*;
use common::memory::*;
use common::random::*;
use common::string::*;

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

    pub unsafe fn calculate(&mut self, ptr: usize, len: usize){
        self.data = 0;

        let sum = Checksum::sum(ptr, len);

        self.data = Checksum::compile(sum);
    }

    pub unsafe fn sum(mut ptr: usize, mut len: usize) -> usize{
        let mut sum = 0;

        while len > 1 {
            sum += *(ptr as *const u16) as usize;
            len -= 2;
            ptr += 2;
        }

        if len > 0 {
            sum += *(ptr as *const u8) as usize;
        }

        return sum;
    }

    pub unsafe fn compile(mut sum: usize) -> u16{
        while (sum >> 16) > 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        return 0xFFFF - (sum as u16);
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
    pub dst: IPv4Addr,
    //Split to fix problem with rust's copy/clone method
    pub options_a: [u8; 20],
    pub options_b: [u8; 20]
}

impl IPv4 {
    pub fn hlen(&self) -> usize{
        return ((self.ver_hlen & 0xF) << 2) as usize;
    }

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
    //Split to fix problem with rust's copy/clone method
    pub options_a: [u8; 20],
    pub options_b: [u8; 20]
}

impl TCP {
    pub fn hlen(&self) -> usize{
        return ((self.flags & 0xF0) >> 2) as usize;
    }

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
            segment_len: n16::new(packet.len.get() - packet.hlen() as u16),
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

        segment.checksum.calculate(segment_addr, packet.len.get() as usize - packet.hlen());

        let packet_addr: *const IPv4 = packet;
        let packet_hlen = packet.hlen();
        packet.checksum.calculate(packet_addr as usize, packet_hlen);

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

        if segment.flags & (1 << 9) != 0 {
            d("            HTTP SYN\n");
            let id = packet.id.get();
            packet.id.set(id + 1);

            segment.flags = segment.flags | (1 << 12);
            segment.ack_num.set(segment.sequence.get() + 1);
            segment.sequence.set(rand() as u32);

            segment.checksum.data = 0;

            let tcpip_psuedo = TCPIPv4Psuedo::new(packet, segment);
            let tcpip_psuedo_addr: *const TCPIPv4Psuedo = &tcpip_psuedo;
            segment.checksum.calculate(tcpip_psuedo_addr as usize, 12 + tcpip_psuedo.segment_len.get() as usize);

            let packet_hlen = packet.hlen();
            packet.checksum.calculate(packet_addr as usize, packet_hlen);

            device.send(frame_addr as usize, 74);
        }else if segment.flags & (1 << 11) != 0{
            d("            HTTP PSH\n");

            // TODO: Allocate space
            let request = String::from_c_slice(slice::from_raw_parts((segment_addr + segment.hlen()) as *const u8, packet.len.get() as usize - packet.hlen() - segment.hlen()));

            let mut request_first = "".to_string();
            for line in request.split("\r\n".to_string()) {
                request_first = line;
                break;
            }

            let message = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n<b>Hello from Redox</b><br/>Your request:<br />".to_string() +
                            request_first + "<br />".to_string() +
                            String::from_num(rand());

            {
                let id = packet.id.get();
                packet.id.set(id + 1);

                segment.flags = segment.flags & (0xFFFF - (1 << 11));
                let sequence = segment.ack_num.get();
                let ack_num = segment.sequence.get() + (packet.len.get() as usize - packet.hlen() - segment.hlen()) as u32;
                segment.ack_num.set(ack_num);
                segment.sequence.set(sequence);

                let packet_len = (packet.hlen() + segment.hlen()) as u16;
                packet.len.set(packet_len);

                segment.checksum.data = 0;

                let tcpip_psuedo = TCPIPv4Psuedo::new(packet, segment);
                let tcpip_psuedo_addr: *const TCPIPv4Psuedo = &tcpip_psuedo;
                segment.checksum.calculate(tcpip_psuedo_addr as usize, 12 + tcpip_psuedo.segment_len.get() as usize);

                let packet_hlen = packet.hlen();
                packet.checksum.calculate(packet_addr as usize, packet_hlen);

                device.send(frame_addr as usize, size_of::<EthernetII>() + packet.len.get() as usize);
            }

            {
                let id = packet.id.get();
                packet.id.set(id + 1);

                segment.flags = segment.flags | (1 << 11) | (1 << 8);

                let packet_len = (packet.hlen() + segment.hlen() + message.len()) as u16;
                packet.len.set(packet_len);

                for i in 0..message.len() {
                    *((segment_addr + segment.hlen() + i) as *mut u8) = message[i] as u8;
                }

                segment.checksum.data = 0;

                let proto = n16::new(packet.proto as u16);
                let segment_hlen = segment.hlen();
                let segment_len = n16::new(segment_hlen as u16 + message.len() as u16);

                segment.checksum.data = Checksum::compile(
                                        Checksum::sum((&packet.src as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                                        Checksum::sum((&packet.dst as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                                        Checksum::sum((&proto as *const n16) as usize, size_of::<n16>()) +
                                        Checksum::sum((&segment_len as *const n16) as usize, size_of::<n16>()) +
                                        Checksum::sum(segment_addr as usize, segment_hlen) +
                                        Checksum::sum(segment_addr as usize + segment_hlen, message.len())
                                    );

                let packet_hlen = packet.hlen();
                packet.checksum.calculate(packet_addr as usize, packet_hlen);

                device.send(frame_addr as usize, size_of::<EthernetII>() + packet.len.get() as usize);
            }
        }else if segment.flags & (1 << 8) != 0 {
            let id = packet.id.get();
            packet.id.set(id + 1);

            segment.checksum.data = 0;

            let tcpip_psuedo = TCPIPv4Psuedo::new(packet, segment);
            let tcpip_psuedo_addr: *const TCPIPv4Psuedo = &tcpip_psuedo;
            segment.checksum.calculate(tcpip_psuedo_addr as usize, 12 + tcpip_psuedo.segment_len.get() as usize);

            let packet_hlen = packet.hlen();
            packet.checksum.calculate(packet_addr as usize, packet_hlen);

            device.send(frame_addr as usize, size_of::<EthernetII>() + packet.len.get() as usize);
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

unsafe fn network_arp(packet: ARP) -> Option<ARP>{
    d("    ");
    packet.d();
    dl();

    if packet.dst_ip.equals(IP_ADDR) {
        if packet.oper.get() == 1 {
            d("        ARP Reply\n");
            let mut response = packet;
            response.oper.set(2);
            response.dst_mac = response.src_mac;
            response.dst_ip = response.src_ip;
            response.src_mac = MAC_ADDR;
            response.src_ip = IP_ADDR;

            return Option::Some(response);
        }else{
            d("        Ignore ARP: Unknown operation\n");
        }
    }else{
        d("        Ignore ARP: Wrong destination\n");
    }

    return Option::None;
}

pub unsafe fn network_frame(device: &NetworkDevice, frame_addr: usize, frame_len: usize){
    let frame = &mut *(frame_addr as *mut EthernetII);
    frame.d();
    dl();

    if frame._type.get() == 0x0800 {
        network_ipv4(device, frame, frame_addr + size_of::<EthernetII>());
    }else if frame._type.get() == 0x0806 {
        let packet = *((frame_addr + size_of::<EthernetII>()) as *const ARP);
        match network_arp(packet){
            Option::Some(response) => {
                let response_addr = alloc(size_of::<EthernetII>() + size_of::<ARP>());

                *(response_addr as *mut EthernetII) = EthernetII {
                    src: MAC_ADDR,
                    dst: frame.src,
                    _type: frame._type
                };

                *((response_addr + size_of::<EthernetII>()) as *mut ARP) = response;

                device.send(response_addr, size_of::<EthernetII>() + size_of::<ARP>());

                unalloc(response_addr);
            },
            Option::None => ()
        }
    }else if frame._type.get() == 0x86DD {
        //Ignore ipv6 for now network_ipv6(device, frame, frame_addr + size_of::<EthernetII>());
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
