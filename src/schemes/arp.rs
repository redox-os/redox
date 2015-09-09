use network::arp::*;
use network::common::*;
use network::ethernet::*;

use programs::common::*;

pub struct ARPResource {
    link: Box<Resource>
}

impl Resource for ARPResource {
    fn url(&self) -> URL {
        return URL::from_string(&"arp://".to_string());
    }

    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        d("TODO: Implement read for arp://\n");
        return Option::None;
    }

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        loop {
            let mut bytes: Vec<u8> = Vec::new();
            match self.link.read_to_end(&mut bytes) {
                Option::Some(_) => {
                    if let Option::Some(packet) = ARP::from_bytes(bytes) {
                        vec.push_all(&packet.data);
                        return Option::Some(packet.data.len());
                    }
                },
                Option::None => return Option::None
            }

            sys_yield();
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        d("TODO: Implement write for arp://\n");
        return Option::None;
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None;
    }

    fn flush(&mut self) -> bool {
        return self.link.flush();
    }
}

pub struct ARPScheme;

impl SessionItem for ARPScheme {
    fn scheme(&self) -> String {
        return "arp".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource>{
        return box ARPResource {
            link: URL::from_string(&"ethernet:///806".to_string()).open()
        };
    }
}

impl ARPScheme {
    pub fn reply_loop(){
        let mut network = URL::from_string(&"network://".to_string()).open();
        loop {
            let mut bytes: Vec<u8> = Vec::new();
            match network.read_to_end(&mut bytes) {
                Option::Some(_) => {
                    if let Option::Some(frame) = EthernetII::from_bytes(bytes) {
                        if frame.header.ethertype.get() == 0x806 && (frame.header.dst.equals(MAC_ADDR) || frame.header.dst.equals(BROADCAST_MAC_ADDR)) {
                            if let Option::Some(packet) = ARP::from_bytes(frame.data) {
                                if packet.header.oper.get() == 1 && packet.header.dst_ip.equals(IP_ADDR) {
                                    let mut response = ARP {
                                        header: packet.header,
                                        data: packet.data.clone()
                                    };
                                    response.header.oper.set(2);
                                    response.header.dst_mac = packet.header.src_mac;
                                    response.header.dst_ip = packet.header.src_ip;
                                    response.header.src_mac = MAC_ADDR;
                                    response.header.src_ip = IP_ADDR;

                                    network.write(EthernetII {
                                        header: EthernetIIHeader {
                                            src: MAC_ADDR,
                                            dst: frame.header.src,
                                            ethertype: frame.header.ethertype
                                        },
                                        data: response.to_bytes()
                                    }.to_bytes().as_slice());
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
