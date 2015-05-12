use core::mem::size_of;

use common::debug::*;
use common::memory::*;
use common::pio::*;

use network::network::*;

pub struct RTL8139 {
    pub base: usize,
    pub memory_mapped: bool
}

static mut RTL8139_TX: u16 = 0;

impl RTL8139 {
    pub unsafe fn send(&self, ptr: usize, len: usize){
        d("RTL8139 send ");
        dd(RTL8139_TX as usize);
        dl();

        let base = self.base as u16;

        outd(base + 0x20 + RTL8139_TX*4, ptr as u32);
        outd(base + 0x10 + RTL8139_TX*4, len as u32 & 0x1FFF);

        RTL8139_TX = (RTL8139_TX + 1) % 4;
    }

    pub unsafe fn handle(&self){
        d("RTL8139 handle");

        let base = self.base as u16;

        let receive_buffer = ind(base + 0x30) as usize;
        let mut capr = (inw(base + 0x38) + 16) as usize;
        let cbr = inw(base + 0x3A) as usize;
        while capr != cbr {
            d(" CAPR ");
            dd(capr);
            d(" CBR ");
            dd(cbr);

            d(" len ");
            let frame_len = *((receive_buffer + capr + 2) as *const u16) as usize;
            dd(frame_len);
            dl();

            let frame_addr = receive_buffer + capr + 4;
            let frame = &mut *(frame_addr as *mut EthernetII);
            frame.d();
            dl();

            if frame._type.get() == 0x0800 {
                let packet = &mut *((frame_addr + 14) as *mut IPv4);
                d("    ");
                packet.d();
                dl();

                if packet.proto == 0x01 {
                    let segment = &mut *((frame_addr + 14 + ((packet.ver_hlen & 0xF) as usize) * 4) as *mut ICMP);
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
                        segment._type = 0x00;

                        segment.checksum.calculate(frame_addr + 14 + ((packet.ver_hlen & 0xF) as usize) * 4, 98 - 14 - ((packet.ver_hlen & 0xF) as usize) * 4);
                        packet.checksum.calculate(frame_addr + 14, 98 - 14);

                        self.send(frame_addr, 98);
                    }else{
                        d("            Ignore ICMP\n");
                    }
                }else if packet.proto == 0x06 {
                    let mut segment = &mut *((frame_addr + 14 + ((packet.ver_hlen & 0xF) as usize) * 4) as *mut TCP);
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

                        if segment.flags & (1 << 12) == 0 {
                            d("            HTTP SYN\n");
                            segment.flags = segment.flags | (1 << 12);
                            segment.ack_num.set(segment.sequence.get() + 1);
                            segment.sequence.set(0x76543210); // TODO: Randomize
                        }else{
                            d("            HTTP ACK\n");
                        }

                        segment.checksum.data = 0;

                        let tcpip_psuedo = TCPIPv4Psuedo::new(packet, segment);
                        let tcpip_psuedo_addr: *const TCPIPv4Psuedo = &tcpip_psuedo;
                        segment.checksum.calculate(tcpip_psuedo_addr as usize, size_of::<TCPIPv4Psuedo>());

                        packet.checksum.calculate(frame_addr + 14, 74 - 14);

                        self.send(frame_addr, 74);
                    }
                }else if packet.proto == 0x11 {
                    let segment = &*((frame_addr + 14 + ((packet.ver_hlen & 0xF) as usize) * 4) as *const UDP);
                    d("        ");
                    segment.d();
                    dl();
                }
            }else if frame._type.get() == 0x0806 {
                let packet = &mut *((frame_addr + 14) as *mut ARP);
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

                    self.send(frame_addr, 42);
                }else{
                    d("        Ignore ARP\n");
                }
            }else if frame._type.get() == 0x86DD {
                let packet = &*((frame_addr + 14) as *const IPv6);
                d("    ");
                packet.d();
                dl();

                if packet.next_header == 0x11 {
                    let segment = &*((frame_addr + 14 + 40) as *const UDP);
                    d("        ");
                    segment.d();
                    dl();
                }
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

            capr = capr + frame_len + 4;
            capr = (capr + 3) & (0xFFFFFFFF - 3);
            if capr >= 8192 {
                capr -= 8192
            }

            outw(base + 0x38, (capr as u16) - 16);
        }

        outw(base + 0x3E, 0x0001);
    }

    pub unsafe fn init(&self){
        RTL8139_TX = 0;

        d("RTL8139 on: ");
        dh(self.base);
        if self.memory_mapped {
            d(" memory mapped");
        }else{
            d(" port mapped");
        }
        dl();

        let base = self.base as u16;

        outb(base + 0x52, 0x00);

        outb(base + 0x37, 0x10);
        while inb(base + 0x37) & 0x10 != 0 {
        }

        let receive_buffer = alloc(10240);
        outd(base + 0x30, receive_buffer as u32);
        outw(base + 0x38, 0);
        outw(base + 0x3A, 0);

        outw(base + 0x3C, 0x0001);

        outd(base + 0x44, 0xf | (1 << 7));

        outb(base + 0x37, 0x0C);
    }
}
