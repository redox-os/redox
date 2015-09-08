use network::common::*;
use network::ethernet::*;

use programs::common::*;

pub struct EthernetResource {
    network: Box<Resource>,
    ethertype: Option<u16>
}

impl Resource for EthernetResource {
    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        d("TODO: Implement read for ethernet://\n");
        return Option::None;
    }

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        loop {
            let mut bytes: Vec<u8> = Vec::new();
            match self.network.read_to_end(&mut bytes) {
                Option::Some(_) => {
                    if let Option::Some(frame) = EthernetII::from_bytes(bytes) {
                        let matches;
                        match self.ethertype {
                            Option::Some(ethertype) => {
                                if ethertype == frame.header.ethertype.get() {
                                    matches = true;
                                }else{
                                    matches = false;
                                }
                            },
                            Option::None => matches = true
                        }

                        if matches {
                            vec.push_all(&frame.data);
                            return Option::Some(frame.data.len());
                        }
                    }
                },
                Option::None => return Option::None
            }

            sys_yield();
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        d("TODO: Implement write for ethernet://\n");
        return Option::None;
    }

    fn flush(&mut self) -> bool {
        return self.network.flush();
    }
}

pub struct EthernetScheme;

impl SessionItem for EthernetScheme {
    fn scheme(&self) -> String {
        return "ethernet".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource>{
        let ethertype;
        if url.path.len() > 0 {
            ethertype = Option::Some(url.path.to_num_radix(16) as u16);
        }else{
            ethertype = Option::None;
        }

        return box EthernetResource {
            network: URL::from_string(&"network://".to_string()).open(),
            ethertype: ethertype
        };
    }
}
