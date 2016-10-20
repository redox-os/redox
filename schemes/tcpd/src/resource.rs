use std::{cmp, mem};
use std::cell::UnsafeCell;
use std::sync::Arc;

use netutils::{n16, n32, Ipv4Addr, Checksum, Tcp, TcpHeader, TCP_SYN, TCP_PSH, TCP_FIN, TCP_ACK};
use resource_scheme::Resource;
use syscall;
use syscall::error::*;

pub struct TcpStream {
    pub ip: usize,
    pub host_addr: Ipv4Addr,
    pub peer_addr: Ipv4Addr,
    pub peer_port: u16,
    pub host_port: u16,
    pub sequence: u32,
    pub acknowledge: u32,
    pub finished: bool
}

impl TcpStream {
    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let path_string = format!("tcp:{}:{}/{}", self.peer_addr.to_string(), self.peer_port, self.host_port);
        let path = path_string.as_bytes();

        for (b, p) in buf.iter_mut().zip(path.iter()) {
            *b = *p;
        }

        Ok(cmp::min(buf.len(), path.len()))
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.finished {
            return Ok(0);
        }

        loop {
            let mut bytes = [0; 65536];
            let count = try!(syscall::read(self.ip, &mut bytes));

            if let Some(segment) = Tcp::from_bytes(&bytes[..count]) {
                if segment.header.dst.get() == self.host_port && segment.header.src.get() == self.peer_port {
                    //println!("Read: {}=={} {:X}: {}", segment.header.sequence.get(), self.acknowledge, segment.header.flags.get(), segment.data.len());

                    if self.acknowledge == segment.header.sequence.get() {
                        if segment.header.flags.get() & TCP_FIN == TCP_FIN {
                            self.finished = true;
                        }

                        if segment.header.flags.get() & (TCP_SYN | TCP_ACK) == TCP_ACK {
                            let flags = if self.finished {
                                TCP_ACK | TCP_FIN
                            } else {
                                TCP_ACK
                            };

                            // Send ACK
                            self.acknowledge += segment.data.len() as u32;
                            let mut tcp = Tcp {
                                header: TcpHeader {
                                    src: n16::new(self.host_port),
                                    dst: n16::new(self.peer_port),
                                    sequence: n32::new(self.sequence),
                                    ack_num: n32::new(self.acknowledge),
                                    flags: n16::new(((mem::size_of::<TcpHeader>() << 10) & 0xF000) as u16 | flags),
                                    window_size: n16::new(65535),
                                    checksum: Checksum {
                                        data: 0
                                    },
                                    urgent_pointer: n16::new(0)
                                },
                                options: Vec::new(),
                                data: Vec::new()
                            };

                            tcp.checksum(&self.host_addr, &self.peer_addr);

                            //println!("Sending read ack: {} {} {:X}", tcp.header.sequence.get(), tcp.header.ack_num.get(), tcp.header.flags.get());

                            let _ = syscall::write(self.ip, &tcp.to_bytes());

                            // TODO: Support broken packets (one packet in two buffers)
                            let mut i = 0;
                            while i < buf.len() && i < segment.data.len() {
                                buf[i] = segment.data[i];
                                i += 1;
                            }
                            return Ok(i);
                        }
                    } else {
                        println!("TCP: MISMATCH: {}=={}", segment.header.sequence.get(), self.acknowledge);
                    }
                } else {
                    println!("TCP: WRONG PORT {}=={} && {}=={}", segment.header.dst.get(), self.host_port, segment.header.src.get(), self.peer_port);
                }
            }
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let tcp_data = Vec::from(buf);

        let mut tcp = Tcp {
            header: TcpHeader {
                src: n16::new(self.host_port),
                dst: n16::new(self.peer_port),
                sequence: n32::new(self.sequence),
                ack_num: n32::new(self.acknowledge),
                flags: n16::new((((mem::size_of::<TcpHeader>()) << 10) & 0xF000) as u16 | TCP_PSH |
                                TCP_ACK),
                window_size: n16::new(65535),
                checksum: Checksum { data: 0 },
                urgent_pointer: n16::new(0),
            },
            options: Vec::new(),
            data: tcp_data,
        };

        tcp.checksum(&self.host_addr, &self.peer_addr);

        match syscall::write(self.ip, &tcp.to_bytes()) {
            Ok(size) => {
                loop {
                    // Wait for ACK
                    let mut bytes = [0; 65536];
                    match syscall::read(self.ip, &mut bytes) {
                        Ok(count) => {
                            if let Some(segment) = Tcp::from_bytes(&bytes[..count]) {
                                if segment.header.dst.get() == self.host_port &&
                                   segment.header.src.get() == self.peer_port {
                                    return if (segment.header.flags.get() & (TCP_SYN | TCP_ACK)) == TCP_ACK {
                                        self.sequence = segment.header.ack_num.get();
                                        self.acknowledge = segment.header.sequence.get();
                                        Ok(size)
                                    } else {
                                        Err(Error::new(EPIPE))
                                    };
                                }
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
            }
            Err(err) => Err(err),
        }
    }

    fn sync(&mut self) -> Result<usize> {
        syscall::fsync(self.ip)
    }

    /// Etablish client
    pub fn client_establish(&mut self) -> bool {
        // Send SYN
        let mut tcp = Tcp {
            header: TcpHeader {
                src: n16::new(self.host_port),
                dst: n16::new(self.peer_port),
                sequence: n32::new(self.sequence),
                ack_num: n32::new(self.acknowledge),
                flags: n16::new(((mem::size_of::<TcpHeader>() << 10) & 0xF000) as u16 | TCP_SYN),
                window_size: n16::new(65535),
                checksum: Checksum { data: 0 },
                urgent_pointer: n16::new(0),
            },
            options: Vec::new(),
            data: Vec::new(),
        };

        tcp.checksum(&self.host_addr, &self.peer_addr);

        match syscall::write(self.ip, &tcp.to_bytes()) {
            Ok(_) => {
                loop {
                    // Wait for SYN-ACK
                    let mut bytes = [0; 65536];
                    match syscall::read(self.ip, &mut bytes) {
                        Ok(count) => {
                            if let Some(segment) = Tcp::from_bytes(&bytes[..count]) {
                                if segment.header.dst.get() == self.host_port &&
                                   segment.header.src.get() == self.peer_port {
                                    return if segment.header.flags.get() & (TCP_SYN | TCP_ACK) == TCP_SYN | TCP_ACK {
                                        self.sequence = segment.header.ack_num.get();
                                        self.acknowledge = segment.header.sequence.get();

                                        self.acknowledge += 1;
                                        tcp = Tcp {
                                            header: TcpHeader {
                                                src: n16::new(self.host_port),
                                                dst: n16::new(self.peer_port),
                                                sequence: n32::new(self.sequence),
                                                ack_num: n32::new(self.acknowledge),
                                                flags: n16::new(((mem::size_of::<TcpHeader>() << 10) & 0xF000) as u16 | TCP_ACK),
                                                window_size: n16::new(65535),
                                                checksum: Checksum {
                                                    data: 0
                                                },
                                                urgent_pointer: n16::new(0)
                                            },
                                            options: Vec::new(),
                                            data: Vec::new()
                                        };

                                        tcp.checksum(&self.host_addr, &self.peer_addr);

                                        let _ = syscall::write(self.ip, &tcp.to_bytes());

                                        true
                                    } else {
                                        false
                                    };
                                }
                            }
                        }
                        Err(_) => return false,
                    }
                }
            }
            Err(_) => false,
        }
    }

    /// Try to establish a server connection
    pub fn server_establish(&mut self, _: Tcp) -> bool {
        // Send SYN-ACK
        self.acknowledge += 1;
        let mut tcp = Tcp {
            header: TcpHeader {
                src: n16::new(self.host_port),
                dst: n16::new(self.peer_port),
                sequence: n32::new(self.sequence),
                ack_num: n32::new(self.acknowledge),
                flags: n16::new(((mem::size_of::<TcpHeader>() << 10) & 0xF000) as u16 | TCP_SYN |
                                TCP_ACK),
                window_size: n16::new(65535),
                checksum: Checksum { data: 0 },
                urgent_pointer: n16::new(0),
            },
            options: Vec::new(),
            data: Vec::new(),
        };

        tcp.checksum(&self.host_addr, &self.peer_addr);

        match syscall::write(self.ip, &tcp.to_bytes()) {
            Ok(_) => {
                loop {
                    // Wait for ACK
                    let mut bytes = [0; 65536];
                    match syscall::read(self.ip, &mut bytes) {
                        Ok(count ) => {
                            if let Some(segment) = Tcp::from_bytes(&bytes[..count]) {
                                if segment.header.dst.get() == self.host_port &&
                                   segment.header.src.get() == self.peer_port {
                                    return if segment.header.flags.get() & (TCP_SYN | TCP_ACK) == TCP_ACK {
                                        self.sequence = segment.header.ack_num.get();
                                        self.acknowledge = segment.header.sequence.get();
                                        true
                                    } else {
                                        false
                                    };
                                }
                            }
                        }
                        Err(_) => return false,
                    }
                }
            }
            Err(_) => false,
        }
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        // Send FIN-ACK
        let mut tcp = Tcp {
            header: TcpHeader {
                src: n16::new(self.host_port),
                dst: n16::new(self.peer_port),
                sequence: n32::new(self.sequence),
                ack_num: n32::new(self.acknowledge),
                flags: n16::new((((mem::size_of::<TcpHeader>()) << 10) & 0xF000) as u16 | TCP_FIN | TCP_ACK),
                window_size: n16::new(65535),
                checksum: Checksum { data: 0 },
                urgent_pointer: n16::new(0),
            },
            options: Vec::new(),
            data: Vec::new(),
        };

        tcp.checksum(&self.host_addr, &self.peer_addr);

        let _ = syscall::write(self.ip, &tcp.to_bytes());
        let _ = syscall::close(self.ip);
    }
}

/// A TCP resource
pub struct TcpResource {
    pub stream: Arc<UnsafeCell<TcpStream>>
}

impl Resource for TcpResource {
    fn dup(&self) -> Result<Box<TcpResource>> {
        Ok(Box::new(TcpResource {
            stream: self.stream.clone()
        }))
    }

    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        unsafe { (*self.stream.get()).path(buf) }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        unsafe { (*self.stream.get()).read(buf) }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        unsafe { (*self.stream.get()).write(buf) }
    }

    fn sync(&mut self) -> Result<usize> {
        unsafe { (*self.stream.get()).sync() }
    }
}
