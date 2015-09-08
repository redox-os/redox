use network::common::*;
use network::ipv4::*;
use network::tcp::*;

use programs::common::*;

pub struct TCPResource {
    link: Box<Resource>,
    port: Option<u16>
}

impl Resource for TCPResource {
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
            match self.link.read_to_end(&mut bytes) {
                Option::Some(_) => {
                    if let Option::Some(packet) = IPv4::from_bytes(bytes) {
                        if packet.header.proto == 0x6 {
                            if let Option::Some(segment) = TCP::from_bytes_ipv4(packet.data, packet.header.src, packet.header.dst) {
                                let matches;
                                match self.port {
                                    Option::Some(port) => {
                                        if port == segment.header.src.get() {
                                            matches = true;
                                        }else{
                                            matches = false;
                                        }
                                    },
                                    Option::None => matches = true
                                }

                                if matches {
                                    vec.push_all(&segment.data);
                                    return Option::Some(segment.data.len());
                                }
                            }
                        }
                    }
                },
                Option::None => return Option::None
            }

            sys_yield();
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        d("TODO: Implement write for tcp://\n");
        return Option::None;
    }

    fn flush(&mut self) -> bool {
        return self.link.flush();
    }
}

pub struct TCPScheme;

impl SessionItem for TCPScheme {
    fn scheme(&self) -> String {
        return "tcp".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource>{
        let port;
        if url.path.len() > 0 {
            port = Option::Some(url.path.to_num() as u16);
        }else{
            port = Option::None;
        }

        return box TCPResource {
            link: URL::from_string(&"ethernet:///800".to_string()).open(),
            port: port
        };
    }
}
