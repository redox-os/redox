use core::mem::swap;

use network::common::*;
use network::ipv4::*;
use network::tcp::*;

use programs::common::*;

pub struct TCPResource {
    ip: Box<Resource>,
    peer_addr: IPv4Addr,
    peer_port: u16,
    host_port: u16,
    sequence: u32,
    acknowledge: u32
}

impl Resource for TCPResource {
    fn url(&self) -> URL {
        return URL::from_string(&("tcp://".to_string() + self.peer_addr.to_string() + ':' + self.peer_port as usize + '/' + self.host_port as usize));
    }

    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        d("TODO: Implement read for tcp://\n");
        return Option::None;
    }

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        loop {
            let mut bytes: Vec<u8> = Vec::new();
            match self.ip.read_to_end(&mut bytes) {
                Option::Some(_) => {
                    if let Option::Some(segment) = TCP::from_bytes(bytes) {
                        if (segment.header.flags.get() & (TCP_PSH | TCP_SYN | TCP_ACK)) == (TCP_PSH | TCP_ACK) && segment.header.src.get() == self.peer_port && segment.header.dst.get() == self.host_port {
                            //Send ACK
                            self.sequence = segment.header.ack_num.get();
                            self.acknowledge = segment.header.sequence.get() + segment.data.len() as u32;
                            let mut tcp = TCP {
                                header: TCPHeader {
                                    src: n16::new(self.host_port),
                                    dst: n16::new(self.peer_port),
                                    sequence: n32::new(self.sequence),
                                    ack_num: n32::new(self.acknowledge),
                                    flags: n16::new((((size_of::<TCPHeader>() + segment.options.len()) << 10) & 0xF000) as u16 | TCP_ACK),
                                    window_size: n16::new(segment.header.window_size.get()),
                                    checksum: Checksum {
                                        data: 0
                                    },
                                    urgent_pointer: n16::new(0)
                                },
                                options: segment.options,
                                data: Vec::new()
                            };

                            unsafe{
                                let proto = n16::new(0x06);
                                let segment_len = n16::new((size_of::<TCPHeader>() + tcp.options.len() + tcp.data.len()) as u16);
                                tcp.header.checksum.data = Checksum::compile(
                                    Checksum::sum((&IP_ADDR as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                                    Checksum::sum((&self.peer_addr as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                                    Checksum::sum((&proto as *const n16) as usize, size_of::<n16>()) +
                                    Checksum::sum((&segment_len as *const n16) as usize, size_of::<n16>()) +
                                    Checksum::sum((&tcp.header as *const TCPHeader) as usize, size_of::<TCPHeader>()) +
                                    Checksum::sum(tcp.options.as_ptr() as usize, tcp.options.len()) +
                                    Checksum::sum(tcp.data.as_ptr() as usize, tcp.data.len())
                                );
                            }

                            self.ip.write(&tcp.to_bytes().as_slice());

                            vec.push_all(&segment.data);
                            return Option::Some(segment.data.len());
                        }
                    }
                },
                Option::None => return Option::None
            }

            sys_yield();
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        let tcp_data = unsafe { Vec::from_raw_buf(buf.as_ptr(), buf.len()) };

        let mut tcp = TCP {
            header: TCPHeader {
                src: n16::new(self.host_port),
                dst: n16::new(self.peer_port),
                sequence: n32::new(self.sequence),
                ack_num: n32::new(self.acknowledge),
                flags: n16::new((((size_of::<TCPHeader>()) << 10) & 0xF000) as u16 | TCP_PSH | TCP_ACK),
                window_size: n16::new(229), //TODO: Use connection window size
                checksum: Checksum {
                    data: 0
                },
                urgent_pointer: n16::new(0)
            },
            options: Vec::new(), //TODO: Use connection options
            data: tcp_data
        };

        unsafe{
            let proto = n16::new(0x06);
            let segment_len = n16::new((size_of::<TCPHeader>() + tcp.data.len()) as u16);
            tcp.header.checksum.data = Checksum::compile(
                Checksum::sum((&IP_ADDR as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                Checksum::sum((&self.peer_addr as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                Checksum::sum((&proto as *const n16) as usize, size_of::<n16>()) +
                Checksum::sum((&segment_len as *const n16) as usize, size_of::<n16>()) +
                Checksum::sum((&tcp.header as *const TCPHeader) as usize, size_of::<TCPHeader>()) +
                Checksum::sum(tcp.options.as_ptr() as usize, tcp.options.len()) +
                Checksum::sum(tcp.data.as_ptr() as usize, tcp.data.len())
            );
        }

        match self.ip.write(&tcp.to_bytes().as_slice()) {
            Option::Some(size) => loop { // Wait for ACK
                let mut bytes: Vec<u8> = Vec::new();
                match self.ip.read_to_end(&mut bytes) {
                    Option::Some(_) => {
                        if let Option::Some(segment) = TCP::from_bytes(bytes) {
                            if segment.header.dst.get() == self.host_port {
                                if (segment.header.flags.get() & (TCP_PSH | TCP_SYN | TCP_ACK)) == TCP_ACK {
                                    self.sequence = segment.header.ack_num.get();
                                    self.acknowledge = segment.header.sequence.get();
                                    return Option::Some(size);
                                }else{
                                    return Option::None;
                                }
                            }
                        }
                    },
                    Option::None => return Option::None
                }
            },
            Option::None => return Option::None
        }
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None;
    }

    fn flush(&mut self) -> bool {
        return self.ip.flush();
    }
}

impl TCPResource {
    //Try to establish a server connection
    pub fn server_establish(&mut self, syn: TCP) -> bool {
        //Send SYN-ACK
        self.acknowledge += 1;
        let mut tcp = TCP {
            header: TCPHeader {
                src: n16::new(self.host_port),
                dst: n16::new(self.peer_port),
                sequence: n32::new(self.sequence),
                ack_num: n32::new(self.acknowledge),
                flags: n16::new((((size_of::<TCPHeader>() + syn.options.len()) << 10) & 0xF000) as u16 | TCP_SYN | TCP_ACK),
                window_size: n16::new(syn.header.window_size.get()),
                checksum: Checksum {
                    data: 0
                },
                urgent_pointer: n16::new(0)
            },
            options: syn.options,
            data: Vec::new()
        };

        unsafe{
            let proto = n16::new(0x06);
            let segment_len = n16::new((size_of::<TCPHeader>() + tcp.options.len() + tcp.data.len()) as u16);
            tcp.header.checksum.data = Checksum::compile(
                Checksum::sum((&IP_ADDR as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                Checksum::sum((&self.peer_addr as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                Checksum::sum((&proto as *const n16) as usize, size_of::<n16>()) +
                Checksum::sum((&segment_len as *const n16) as usize, size_of::<n16>()) +
                Checksum::sum((&tcp.header as *const TCPHeader) as usize, size_of::<TCPHeader>()) +
                Checksum::sum(tcp.options.as_ptr() as usize, tcp.options.len()) +
                Checksum::sum(tcp.data.as_ptr() as usize, tcp.data.len())
            );
        }

        match self.ip.write(&tcp.to_bytes().as_slice()) {
            Option::Some(_) => loop { // Wait for ACK
                let mut bytes: Vec<u8> = Vec::new();
                match self.ip.read_to_end(&mut bytes) {
                    Option::Some(_) => {
                        if let Option::Some(segment) = TCP::from_bytes(bytes) {
                            if segment.header.dst.get() == self.host_port {
                                if (segment.header.flags.get() & (TCP_PSH | TCP_SYN | TCP_ACK)) == TCP_ACK {
                                    self.sequence = segment.header.ack_num.get();
                                    self.acknowledge = segment.header.sequence.get();
                                    return true;
                                }else{
                                    return false;
                                }
                            }
                        }
                    },
                    Option::None => return false
                }
            },
            Option::None => return false
        }
    }
}

impl Drop for TCPResource {
    fn drop(&mut self){
        //Send FIN-ACK
        let mut tcp = TCP {
            header: TCPHeader {
                src: n16::new(self.host_port),
                dst: n16::new(self.peer_port),
                sequence: n32::new(self.sequence),
                ack_num: n32::new(self.acknowledge),
                flags: n16::new((((size_of::<TCPHeader>()) << 10) & 0xF000) as u16 | TCP_FIN | TCP_ACK),
                window_size: n16::new(229),
                checksum: Checksum {
                    data: 0
                },
                urgent_pointer: n16::new(0)
            },
            options: Vec::new(),
            data: Vec::new()
        };

        unsafe{
            let proto = n16::new(0x06);
            let segment_len = n16::new((size_of::<TCPHeader>() + tcp.options.len() + tcp.data.len()) as u16);
            tcp.header.checksum.data = Checksum::compile(
                Checksum::sum((&IP_ADDR as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                Checksum::sum((&self.peer_addr as *const IPv4Addr) as usize, size_of::<IPv4Addr>()) +
                Checksum::sum((&proto as *const n16) as usize, size_of::<n16>()) +
                Checksum::sum((&segment_len as *const n16) as usize, size_of::<n16>()) +
                Checksum::sum((&tcp.header as *const TCPHeader) as usize, size_of::<TCPHeader>()) +
                Checksum::sum(tcp.options.as_ptr() as usize, tcp.options.len()) +
                Checksum::sum(tcp.data.as_ptr() as usize, tcp.data.len())
            );
        }

        match self.ip.write(&tcp.to_bytes().as_slice()) {
            Option::Some(_) => /*loop { // Wait for ACK
                let mut bytes: Vec<u8> = Vec::new();
                match self.ip.read_to_end(&mut bytes) {
                    Option::Some(_) => {
                        if let Option::Some(segment) = TCP::from_bytes(bytes) {
                            if segment.header.dst.get() == self.host_port {
                                if (segment.header.flags.get() & TCP_FIN) == TCP_FIN {
                                    break;
                                }
                            }
                        }
                    },
                    Option::None => ()
                }
            }*/(),
            Option::None => ()
        }
    }
}

pub struct TCPScheme;

impl SessionItem for TCPScheme {
    fn scheme(&self) -> String {
        return "tcp".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource>{
        if url.path().len() > 0 {
            let host_port = url.path().to_num() as u16;

            loop {
                let mut ip = URL::from_string(&"ip:///6".to_string()).open();

                let mut bytes: Vec<u8> = Vec::new();
                match ip.read_to_end(&mut bytes) {
                    Option::Some(_) => {
                        if let Option::Some(segment) = TCP::from_bytes(bytes) {
                            if segment.header.dst.get() == host_port && (segment.header.flags.get() & (TCP_PSH | TCP_SYN | TCP_ACK)) == TCP_SYN {
                                let mut peer_addr = IPv4Addr {
                                    bytes: [0, 0, 0, 0]
                                };

                                let mut i = 0;
                                for part in ip.url().host().split(".".to_string()) {
                                    let octet = part.to_num() as u8;
                                    match i {
                                        0 => peer_addr.bytes[0] = octet,
                                        1 => peer_addr.bytes[1] = octet,
                                        2 => peer_addr.bytes[2] = octet,
                                        3 => peer_addr.bytes[3] = octet,
                                        _ => break
                                    }
                                    i += 1;
                                }

                                let mut ret = box TCPResource {
                                    ip: ip,
                                    peer_addr: peer_addr,
                                    peer_port: segment.header.src.get(),
                                    host_port: host_port,
                                    sequence: rand() as u32,
                                    acknowledge: segment.header.sequence.get()
                                };

                                if ret.server_establish(segment) {
                                    return ret;
                                }
                            }
                        }
                    },
                    Option::None => break
                }
            }
        }else{
            let host_port = (rand() % 32768 + 32768) as u16;
            d("Implement TCP Client Connections: ");
            dd(host_port as usize);
            dl();
        }

        return box NoneResource;
    }
}
