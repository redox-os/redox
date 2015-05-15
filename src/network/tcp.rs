use core::clone::Clone;
use core::mem::size_of;
use core::option::Option;

use common::debug::*;
use common::vector::*;

use network::common::*;

#[derive(Copy, Clone)]
pub struct TCPHeader {
    pub src: n16,
    pub dst: n16,
    pub sequence: n32,
    pub ack_num: n32,
    pub flags: u16,
    pub window_size: n16,
    pub checksum: Checksum,
    pub urgent_pointer: n16
}

pub struct TCP {
    header: TCPHeader,
    options: Vector<u8>,
    data: Vector<u8>
}

impl FromBytes for TCP {
    fn from_bytes(bytes: Vector<u8>) -> Option<TCP> {
        if bytes.len() >= size_of::<TCPHeader>() {
            unsafe {
                let header = *(bytes.data as *const TCPHeader);
                let header_len = ((header.flags & 0xF0) >> 2) as usize;

                return Option::Some(TCP {
                    header: header,
                    options: bytes.sub(size_of::<TCPHeader>(), header_len - size_of::<TCPHeader>()),
                    data: bytes.sub(header_len, bytes.len() - header_len)
                });
            }
        }
        return Option::None;
    }
}

impl ToBytes for TCP {
    fn to_bytes(&self) -> Vector<u8> {
        unsafe{
            let header_ptr: *const TCPHeader = &self.header;
            Vector::<u8>::from_raw(header_ptr as *const u8, size_of::<TCPHeader>()) + self.options.clone() + self.data.clone()
        }
    }
}

impl Response for TCP {
    fn respond(&self) -> Vector<Vector<u8>> {
        d("            ");
        self.d();
        dl();

        /*
        if segment.dst.get() == 80 {
            if cfg!(debug_network){
                d("            HTTP Reply ");
                dh(segment.flags as usize);
                d(" ");
                dh(segment.sequence.get() as usize);
                d(" ");
                dh(segment.ack_num.get() as usize);
                dl();
            }

            frame.dst = frame.src;
            frame.src = MAC_ADDR;
            packet.dst = packet.src;
            packet.src = IP_ADDR;
            segment.dst.set(segment.src.get());
            segment.src.set(80);

            let frame_addr: *const EthernetII = frame;
            let packet_addr: *const IPv4 = packet;

            if segment.flags & (1 << 9) != 0 {
                if cfg!(debug_network){
                    d("            HTTP SYN\n");
                }
                let id = packet.id.get();
                packet.id.set(id + 1);

                segment.flags = segment.flags | (1 << 12);
                segment.ack_num.set(segment.sequence.get() + 1);
                segment.sequence.set(rand() as u32);


                let proto = n16::new(packet.proto as u16);
                let segment_hlen = segment.head_len();
                let segment_len = n16::new(segment_hlen as u16);
                segment.checksum.data = 0;
                segment.checksum.data = Checksum::compile(
                                            Checksum::sum((&packet.src as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                                            Checksum::sum((&packet.dst as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                                            Checksum::sum((&proto as *const n16) as usize, size_of::<n16>()) +
                                            Checksum::sum((&segment_len as *const n16) as usize, size_of::<n16>()) +
                                            Checksum::sum(segment_addr, segment_hlen)
                                        );

                let packet_hlen = packet.head_len();
                packet.checksum.calculate(packet_addr as usize, packet_hlen);

                device.send(frame_addr as usize, size_of::<EthernetII>() + packet.head_len() + segment.head_len());
            }else if segment.flags & (1 << 11) != 0{
                if cfg!(debug_network){
                    d("            HTTP PSH\n");
                }

                let request = String::from_c_slice(slice::from_raw_parts((segment_addr + segment.head_len()) as *const u8, packet.len.get() as usize - packet.head_len() - segment.head_len()));

                {
                    let id = packet.id.get();
                    packet.id.set(id + 1);

                    segment.flags = segment.flags & (0xFFFF - (1 << 11));
                    let sequence = segment.ack_num.get();
                    let ack_num = segment.sequence.get() + (packet.len.get() as usize - packet.head_len() - segment.head_len()) as u32;
                    segment.ack_num.set(ack_num);
                    segment.sequence.set(sequence);

                    let packet_len = (packet.head_len() + segment.head_len()) as u16;
                    packet.len.set(packet_len);

                    let proto = n16::new(packet.proto as u16);
                    let segment_hlen = segment.head_len();
                    let segment_len = n16::new(segment_hlen as u16);
                    segment.checksum.data = 0;
                    segment.checksum.data = Checksum::compile(
                                                Checksum::sum((&packet.src as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                                                Checksum::sum((&packet.dst as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                                                Checksum::sum((&proto as *const n16) as usize, size_of::<n16>()) +
                                                Checksum::sum((&segment_len as *const n16) as usize, size_of::<n16>()) +
                                                Checksum::sum(segment_addr, segment_hlen)
                                            );

                    let packet_hlen = packet.head_len();
                    packet.checksum.calculate(packet_addr as usize, packet_hlen);

                    device.send(frame_addr as usize, size_of::<EthernetII>() + packet.len.get() as usize);
                }

                {
                    let html = http_response(request);

                    let response_len = size_of::<EthernetII>() + packet.head_len() + segment.head_len() + html.len();
                    let response_addr = alloc(response_len);

                    let response_frame = &mut *(response_addr as *mut EthernetII);
                    *response_frame = *frame;

                    let response_packet = &mut *((response_addr + size_of::<EthernetII>()) as *mut IPv4);
                    *response_packet = *packet;
                    response_packet.id.set(packet.id.get() + 1);
                    response_packet.len.set((packet.head_len() + segment.head_len() + html.len()) as u16);
                    response_packet.checksum.calculate(response_addr + size_of::<EthernetII>(), packet.head_len());

                    let response_segment = &mut *((response_addr + size_of::<EthernetII>() + packet.head_len()) as *mut TCP);
                    *response_segment = *segment;
                    response_segment.flags = segment.flags | (1 << 11) | (1 << 8);
                    response_segment.checksum.data = 0;

                    for i in 0..html.len() {
                        *((response_addr + size_of::<EthernetII>() + packet.head_len() + segment.head_len() + i) as *mut u8) = html[i] as u8;
                    }

                    let proto = n16::new(packet.proto as u16);
                    let segment_len = n16::new(segment.head_len() as u16 + html.len() as u16);
                    response_segment.checksum.data = Checksum::compile(
                                                Checksum::sum((&packet.src as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                                                Checksum::sum((&packet.dst as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                                                Checksum::sum((&proto as *const n16) as usize, size_of::<n16>()) +
                                                Checksum::sum((&segment_len as *const n16) as usize, size_of::<n16>()) +
                                                Checksum::sum(response_addr + size_of::<EthernetII>() + packet.head_len(), segment.head_len()) +
                                                Checksum::sum(response_addr + size_of::<EthernetII>() + packet.head_len() + segment.head_len(), html.len())
                                            );

                    device.send(response_addr, response_len);

                    unalloc(response_addr);
                }
            }else if segment.flags & (1 << 8) != 0 {
                if cfg!(debug_network){
                    d("            HTTP FIN\n");
                }
                let id = packet.id.get();
                packet.id.set(id + 1);

                let proto = n16::new(packet.proto as u16);
                let segment_hlen = segment.head_len();
                let segment_len = n16::new(segment_hlen as u16);
                segment.checksum.data = 0;
                segment.checksum.data = Checksum::compile(
                                            Checksum::sum((&packet.src as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                                            Checksum::sum((&packet.dst as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                                            Checksum::sum((&proto as *const n16) as usize, size_of::<n16>()) +
                                            Checksum::sum((&segment_len as *const n16) as usize, size_of::<n16>()) +
                                            Checksum::sum(segment_addr, segment_hlen)
                                        );

                let packet_hlen = packet.head_len();
                packet.checksum.calculate(packet_addr as usize, packet_hlen);

                device.send(frame_addr as usize, size_of::<EthernetII>() + packet.len.get() as usize);
            }
        }
        */

        return Vector::new();
    }
}

impl TCP {
    pub fn d(&self){
        d("TCP from ");
        dd(self.header.src.get() as usize);
        d(" to ");
        dd(self.header.dst.get() as usize);
        d(" options ");
        dd(self.options.len());
        d(" data ");
        dd(self.data.len());
    }
}