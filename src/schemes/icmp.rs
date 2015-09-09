use network::common::*;
use network::ethernet::*;
use network::icmp::*;
use network::ipv4::*;

use programs::common::*;

pub struct ICMPScheme;

impl SessionItem for ICMPScheme {
    fn scheme(&self) -> String {
        return "icmp".to_string();
    }
}

impl ICMPScheme {
    pub fn reply_loop(){
        let mut network = URL::from_string(&"network://".to_string()).open();
        loop {
            let mut bytes: Vec<u8> = Vec::new();
            match network.read_to_end(&mut bytes) {
                Option::Some(_) => {
                    if let Option::Some(frame) = EthernetII::from_bytes(bytes) {
                        if frame.header.ethertype.get() == 0x800 && (frame.header.dst.equals(MAC_ADDR) || frame.header.dst.equals(BROADCAST_MAC_ADDR)) {
                            if let Option::Some(packet) = IPv4::from_bytes(frame.data) {
                                if packet.header.proto == 1 && packet.header.dst.equals(IP_ADDR) {
                                    if let Option::Some(message) = ICMP::from_bytes(packet.data) {
                                        if message.header._type == 0x08 {
                                            let mut response = ICMP {
                                                header: message.header,
                                                data: message.data
                                            };

                                            response.header._type = 0x00;

                                            unsafe{
                                                response.header.checksum.data = 0;

                                                let header_ptr: *const ICMPHeader = &response.header;
                                                response.header.checksum.data = Checksum::compile(
                                                    Checksum::sum(header_ptr as usize, size_of::<ICMPHeader>()) +
                                                    Checksum::sum(response.data.as_ptr() as usize, response.data.len())
                                                );
                                            }

                                            let mut response_packet = IPv4 {
                                                header: packet.header,
                                                options: packet.options,
                                                data: response.to_bytes()
                                            };

                                            response_packet.header.dst = packet.header.src;
                                            response_packet.header.src = IP_ADDR;
                                            response_packet.header.len.set((size_of::<IPv4Header>() + response_packet.options.len() + response_packet.data.len()) as u16);

                                            unsafe{
                                                response_packet.header.checksum.data = 0;

                                                let header_ptr: *const IPv4Header = &response_packet.header;
                                                response_packet.header.checksum.data = Checksum::compile(
                                                    Checksum::sum(header_ptr as usize, size_of::<IPv4Header>()) +
                                                    Checksum::sum(response_packet.options.as_ptr() as usize, response_packet.options.len())
                                                );
                                            }

                                            network.write(EthernetII {
                                                header: EthernetIIHeader {
                                                    src: MAC_ADDR,
                                                    dst: frame.header.src,
                                                    ethertype: frame.header.ethertype
                                                },
                                                data: response_packet.to_bytes()
                                            }.to_bytes().as_slice());
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                Option::None => break
            }

            sys_yield();
        }
    }
}
