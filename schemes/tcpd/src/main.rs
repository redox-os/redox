extern crate event;
extern crate netutils;
extern crate rand;
extern crate syscall;

use rand::{Rng, OsRng};
use std::collections::{BTreeMap, VecDeque};
use std::cell::RefCell;
use std::fs::File;
use std::io::{self, Read, Write};
use std::{mem, slice, str, thread};
use std::os::unix::io::FromRawFd;
use std::rc::Rc;

use event::EventQueue;
use netutils::{n16, n32, Ipv4, Ipv4Addr, Ipv4Header, Tcp, TcpHeader, Checksum, TCP_FIN, TCP_SYN, TCP_RST, TCP_PSH, TCP_ACK};
use syscall::data::Packet;
use syscall::error::{Error, Result, EACCES, EADDRINUSE, EBADF, EINVAL, EISCONN, EMSGSIZE, ENOTCONN, EWOULDBLOCK};
use syscall::flag::{EVENT_READ, O_CREAT, O_RDWR, O_NONBLOCK};
use syscall::scheme::SchemeMut;

fn parse_socket(socket: &str) -> (Ipv4Addr, u16) {
    let mut socket_parts = socket.split(":");
    let host = Ipv4Addr::from_str(socket_parts.next().unwrap_or(""));
    let port = socket_parts.next().unwrap_or("").parse::<u16>().unwrap_or(0);
    (host, port)
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum State {
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
    Closed
}

struct Handle {
    local: (Ipv4Addr, u16),
    remote: (Ipv4Addr, u16),
    flags: usize,
    events: usize,
    state: State,
    seq: u32,
    ack: u32,
    data: VecDeque<(Ipv4, Tcp)>,
    todo_dup: VecDeque<Packet>,
    todo_read: VecDeque<Packet>,
    todo_write: VecDeque<Packet>,
}

impl Handle {
    fn is_connected(&self) -> bool {
        self.remote.0 != Ipv4Addr::NULL && self.remote.1 != 0
    }

    fn read_closed(&self) -> bool {
        self.state == State::CloseWait || self.state == State::LastAck || self.state == State::TimeWait || self.state == State::Closed
    }

    fn matches(&self, ip: &Ipv4, tcp: &Tcp) -> bool {
        // Local address not set or IP dst matches or is broadcast
        (self.local.0 == Ipv4Addr::NULL || ip.header.dst == self.local.0 || ip.header.dst == Ipv4Addr::BROADCAST)
        // Local port matches UDP dst
        && tcp.header.dst.get() == self.local.1
        // Remote address not set or is broadcast, or IP src matches
        && (self.remote.0 == Ipv4Addr::NULL || self.remote.0 == Ipv4Addr::BROADCAST || ip.header.src == self.remote.0)
        // Remote port not set or UDP src matches
        && (self.remote.1 == 0 || tcp.header.src.get() == self.remote.1)
    }

    fn create_tcp(&self, flags: u16, data: Vec<u8>) -> Tcp {
        Tcp {
            header: TcpHeader {
                src: n16::new(self.local.1),
                dst: n16::new(self.remote.1),
                sequence: n32::new(self.seq),
                ack_num: n32::new(self.ack),
                flags: n16::new(((mem::size_of::<TcpHeader>() << 10) & 0xF000) as u16 | (flags & 0xFFF)),
                window_size: n16::new(8192),
                checksum: Checksum { data: 0 },
                urgent_pointer: n16::new(0),
            },
            options: Vec::new(),
            data: data
        }
    }

    fn create_ip(&self, id: u16, data: Vec<u8>) -> Ipv4 {
        Ipv4 {
            header: Ipv4Header {
                ver_hlen: 0x45,
                services: 0,
                len: n16::new((data.len() + mem::size_of::<Ipv4Header>()) as u16),
                id: n16::new(id),
                flags_fragment: n16::new(0),
                ttl: 127,
                proto: 0x06,
                checksum: Checksum { data: 0 },
                src: self.local.0,
                dst: self.remote.0
            },
            options: Vec::new(),
            data: data
        }
    }
}

struct Tcpd {
    scheme_file: File,
    tcp_file: File,
    ports: BTreeMap<u16, usize>,
    next_id: usize,
    handles: BTreeMap<usize, Handle>,
    rng: OsRng,
}

impl Tcpd {
    fn new(scheme_file: File, tcp_file: File) -> Self {
        Tcpd {
            scheme_file: scheme_file,
            tcp_file: tcp_file,
            ports: BTreeMap::new(),
            next_id: 1,
            handles: BTreeMap::new(),
            rng: OsRng::new().expect("tcpd: failed to open RNG")
        }
    }

    fn scheme_event(&mut self) -> io::Result<()> {
        loop {
            let mut packet = Packet::default();
            if self.scheme_file.read(&mut packet)? == 0 {
                break;
            }

            let a = packet.a;
            self.handle(&mut packet);
            if packet.a == (-EWOULDBLOCK) as usize {
                if let Some(mut handle) = self.handles.get_mut(&packet.b) {
                    match a {
                        syscall::number::SYS_DUP => {
                            packet.a = a;
                            handle.todo_dup.push_back(packet);
                        },
                        syscall::number::SYS_READ => {
                            packet.a = a;
                            handle.todo_read.push_back(packet);
                        },
                        syscall::number::SYS_WRITE => {
                            packet.a = a;
                            handle.todo_write.push_back(packet);
                        },
                        _ => {
                            self.scheme_file.write(&packet)?;
                        }
                    }
                }
            } else {
                self.scheme_file.write(&packet)?;
            }
        }

        Ok(())
    }

    fn tcp_event(&mut self) -> io::Result<()> {
        loop {
            let mut bytes = [0; 65536];
            let count = self.tcp_file.read(&mut bytes)?;
            if count == 0 {
                break;
            }
            if let Some(ip) = Ipv4::from_bytes(&bytes[.. count]) {
                if let Some(tcp) = Tcp::from_bytes(&ip.data) {
                    let mut closing = Vec::new();
                    let mut found_connection = false;
                    for (id, handle) in self.handles.iter_mut() {
                        if handle.state != State::Listen && handle.matches(&ip, &tcp) {
                            found_connection = true;

                            match handle.state {
                                State::SynReceived => if tcp.header.flags.get() & (TCP_SYN | TCP_ACK) == TCP_ACK && tcp.header.ack_num.get() == handle.seq {
                                    handle.state = State::Established;
                                },
                                State::SynSent => if tcp.header.flags.get() & (TCP_SYN | TCP_ACK) == TCP_SYN | TCP_ACK && tcp.header.ack_num.get() == handle.seq {
                                    handle.state = State::Established;
                                    handle.ack = tcp.header.sequence.get() + 1;

                                    let tcp = handle.create_tcp(TCP_ACK, Vec::new());
                                    let ip = handle.create_ip(self.rng.gen(), tcp.to_bytes());
                                    self.tcp_file.write(&ip.to_bytes())?;
                                },
                                State::Established => if tcp.header.flags.get() & (TCP_SYN | TCP_ACK) == TCP_ACK && tcp.header.ack_num.get() == handle.seq {
                                    handle.ack = tcp.header.sequence.get();

                                    if ! tcp.data.is_empty() {
                                        handle.data.push_back((ip.clone(), tcp.clone()));
                                        handle.ack += tcp.data.len() as u32;

                                        let tcp = handle.create_tcp(TCP_ACK, Vec::new());
                                        let ip = handle.create_ip(self.rng.gen(), tcp.to_bytes());
                                        self.tcp_file.write(&ip.to_bytes())?;
                                    } else if tcp.header.flags.get() & TCP_FIN == TCP_FIN {
                                        handle.state = State::CloseWait;

                                        handle.ack += 1;

                                        let tcp = handle.create_tcp(TCP_ACK, Vec::new());
                                        let ip = handle.create_ip(self.rng.gen(), tcp.to_bytes());
                                        self.tcp_file.write(&ip.to_bytes())?;
                                    }
                                },
                                //TODO: Time wait
                                State::FinWait1 => if tcp.header.flags.get() & (TCP_SYN | TCP_ACK) == TCP_ACK && tcp.header.ack_num.get() == handle.seq {
                                    handle.ack = tcp.header.sequence.get() + 1;

                                    if tcp.header.flags.get() & TCP_FIN == TCP_FIN {
                                        handle.state = State::TimeWait;

                                        let tcp = handle.create_tcp(TCP_ACK, Vec::new());
                                        let ip = handle.create_ip(self.rng.gen(), tcp.to_bytes());
                                        self.tcp_file.write(&ip.to_bytes())?;

                                        closing.push(*id);
                                    } else {
                                        handle.state = State::FinWait2;
                                    }
                                },
                                State::FinWait2 => if tcp.header.flags.get() & (TCP_SYN | TCP_ACK | TCP_FIN) == TCP_ACK | TCP_FIN && tcp.header.ack_num.get() == handle.seq {
                                    handle.ack = tcp.header.sequence.get() + 1;

                                    handle.state = State::TimeWait;

                                    let tcp = handle.create_tcp(TCP_ACK, Vec::new());
                                    let ip = handle.create_ip(self.rng.gen(), tcp.to_bytes());
                                    self.tcp_file.write(&ip.to_bytes())?;

                                    closing.push(*id);
                                },
                                State::LastAck => if tcp.header.flags.get() & (TCP_SYN | TCP_ACK) == TCP_ACK && tcp.header.ack_num.get() == handle.seq {
                                    handle.state = State::Closed;
                                    closing.push(*id);
                                },
                                _ => ()
                            }

                            while ! handle.todo_read.is_empty() && (! handle.data.is_empty() || handle.read_closed()) {
                                let mut packet = handle.todo_read.pop_front().unwrap();
                                let buf = unsafe { slice::from_raw_parts_mut(packet.c as *mut u8, packet.d) };
                                if let Some((_ip, tcp)) = handle.data.pop_front() {
                                    let mut i = 0;
                                    while i < buf.len() && i < tcp.data.len() {
                                        buf[i] = tcp.data[i];
                                        i += 1;
                                    }
                                    packet.a = i;
                                } else {
                                    packet.a = 0;
                                }

                                self.scheme_file.write(&packet)?;
                            }

                            if ! handle.todo_write.is_empty() && handle.state == State::Established {
                                let mut packet = handle.todo_write.pop_front().unwrap();
                                let buf = unsafe { slice::from_raw_parts(packet.c as *const u8, packet.d) };

                                let tcp = handle.create_tcp(TCP_ACK | TCP_PSH, buf.to_vec());
                                let ip = handle.create_ip(self.rng.gen(), tcp.to_bytes());
                                let result = self.tcp_file.write(&ip.to_bytes()).map_err(|err| err.into_sys());
                                if result.is_ok() {
                                    handle.seq += buf.len() as u32;
                                }
                                packet.a = Error::mux(result.and(Ok(buf.len())));

                                self.scheme_file.write(&packet)?;
                            }

                            if handle.events & EVENT_READ == EVENT_READ {
                                if let Some(&(ref _ip, ref tcp)) = handle.data.get(0) {
                                    self.scheme_file.write(&Packet {
                                        id: 0,
                                        pid: 0,
                                        uid: 0,
                                        gid: 0,
                                        a: syscall::number::SYS_FEVENT,
                                        b: *id,
                                        c: EVENT_READ,
                                        d: tcp.data.len()
                                    })?;
                                }
                            }
                        }
                    }

                    for file in closing {
                        let handle = self.handles.remove(&file).unwrap();

                        let remove = if let Some(mut port) = self.ports.get_mut(&handle.local.1) {
                            *port = *port + 1;
                            *port == 0
                        } else {
                            false
                        };

                        if remove {
                            self.ports.remove(&handle.local.1);
                        }
                    }

                    if ! found_connection && tcp.header.flags.get() & (TCP_SYN | TCP_ACK) == TCP_SYN {
                        let mut new_handles = Vec::new();

                        for (_id, handle) in self.handles.iter_mut() {
                            if handle.state == State::Listen && handle.matches(&ip, &tcp) {
                                handle.data.push_back((ip.clone(), tcp.clone()));

                                while ! handle.todo_dup.is_empty() && ! handle.data.is_empty() {
                                    let mut packet = handle.todo_dup.pop_front().unwrap();
                                    let (ip, tcp) = handle.data.pop_front().unwrap();

                                    let mut new_handle = Handle {
                                        local: handle.local,
                                        remote: (ip.header.src, tcp.header.src.get()),
                                        flags: handle.flags,
                                        events: 0,
                                        state: State::SynReceived,
                                        seq: self.rng.gen(),
                                        ack: tcp.header.sequence.get() + 1,
                                        data: VecDeque::new(),
                                        todo_dup: VecDeque::new(),
                                        todo_read: VecDeque::new(),
                                        todo_write: VecDeque::new(),
                                    };

                                    let tcp = new_handle.create_tcp(TCP_SYN | TCP_ACK, Vec::new());
                                    let ip = new_handle.create_ip(self.rng.gen(), tcp.to_bytes());
                                    self.tcp_file.write(&ip.to_bytes())?;

                                    new_handle.seq += 1;

                                    handle.data.retain(|&(ref ip, ref tcp)| {
                                        if new_handle.matches(ip, tcp) {
                                            false
                                        } else {
                                            true
                                        }
                                    });

                                    if let Some(mut port) = self.ports.get_mut(&handle.local.1) {
                                        *port = *port + 1;
                                    }

                                    let id = self.next_id;
                                    self.next_id += 1;

                                    packet.a = id;

                                    new_handles.push((packet, new_handle));
                                }
                            }
                        }

                        for (packet, new_handle) in new_handles {
                            self.handles.insert(packet.a, new_handle);
                            self.scheme_file.write(&packet)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl SchemeMut for Tcpd {
    fn open(&mut self, url: &[u8], flags: usize, uid: u32, _gid: u32) -> Result<usize> {
        let path = str::from_utf8(url).or(Err(Error::new(EINVAL)))?;

        let mut parts = path.split("/");
        let remote = parse_socket(parts.next().unwrap_or(""));
        let mut local = parse_socket(parts.next().unwrap_or(""));

        if local.1 == 0 {
            local.1 = self.rng.gen_range(32768, 65535);
        }

        if local.1 <= 1024 && uid != 0 {
            return Err(Error::new(EACCES));
        }

        if self.ports.contains_key(&local.1) {
            return Err(Error::new(EADDRINUSE));
        }

        let mut handle = Handle {
            local: local,
            remote: remote,
            flags: flags,
            events: 0,
            state: State::Listen,
            seq: 0,
            ack: 0,
            data: VecDeque::new(),
            todo_dup: VecDeque::new(),
            todo_read: VecDeque::new(),
            todo_write: VecDeque::new(),
        };

        if handle.is_connected() {
            handle.seq = self.rng.gen();
            handle.ack = 0;
            handle.state = State::SynSent;

            let tcp = handle.create_tcp(TCP_SYN, Vec::new());
            let ip = handle.create_ip(self.rng.gen(), tcp.to_bytes());
            self.tcp_file.write(&ip.to_bytes()).map_err(|err| err.into_sys())?;

            handle.seq += 1;
        }

        self.ports.insert(local.1, 1);

        let id = self.next_id;
        self.next_id += 1;

        self.handles.insert(id, handle);

        Ok(id)
    }

    fn dup(&mut self, file: usize, buf: &[u8]) -> Result<usize> {
        let path = str::from_utf8(buf).or(Err(Error::new(EINVAL)))?;

        let handle = {
            let mut handle = self.handles.get_mut(&file).ok_or(Error::new(EBADF))?;

            let mut new_handle = Handle {
                local: handle.local,
                remote: handle.remote,
                flags: handle.flags,
                events: 0,
                state: handle.state,
                seq: handle.seq,
                ack: handle.ack,
                data: VecDeque::new(),
                todo_dup: VecDeque::new(),
                todo_read: VecDeque::new(),
                todo_write: VecDeque::new(),
            };

            if path == "listen" {
                if handle.is_connected() {
                    return Err(Error::new(EISCONN));
                } else if let Some((ip, tcp)) = handle.data.pop_front() {
                    new_handle.remote = (ip.header.src, tcp.header.src.get());

                    new_handle.seq = self.rng.gen();
                    new_handle.ack = tcp.header.sequence.get() + 1;
                    new_handle.state = State::SynReceived;

                    let tcp = new_handle.create_tcp(TCP_SYN | TCP_ACK, Vec::new());
                    let ip = new_handle.create_ip(self.rng.gen(), tcp.to_bytes());
                    self.tcp_file.write(&ip.to_bytes()).map_err(|err| err.into_sys()).and(Ok(buf.len()))?;

                    new_handle.seq += 1;
                } else {
                    return Err(Error::new(EWOULDBLOCK));
                }

                handle.data.retain(|&(ref ip, ref tcp)| {
                    if new_handle.matches(ip, tcp) {
                        false
                    } else {
                        true
                    }
                });
            } else if path.is_empty() {
                new_handle.data = handle.data.clone();
            } else if handle.is_connected() {
                return Err(Error::new(EISCONN));
            } else {
                new_handle.remote = parse_socket(path);

                if new_handle.is_connected() {
                    new_handle.seq = self.rng.gen();
                    new_handle.ack = 0;
                    new_handle.state = State::SynSent;

                    handle.data.retain(|&(ref ip, ref tcp)| {
                        if new_handle.matches(ip, tcp) {
                            new_handle.data.push_back((ip.clone(), tcp.clone()));
                            false
                        } else {
                            true
                        }
                    });

                    let tcp = new_handle.create_tcp(TCP_SYN, Vec::new());
                    let ip = new_handle.create_ip(self.rng.gen(), tcp.to_bytes());
                    self.tcp_file.write(&ip.to_bytes()).map_err(|err| err.into_sys()).and(Ok(buf.len()))?;
                } else {
                    return Err(Error::new(EINVAL));
                }
            }

            new_handle
        };

        if let Some(mut port) = self.ports.get_mut(&handle.local.1) {
            *port = *port + 1;
        }

        let id = self.next_id;
        self.next_id += 1;

        self.handles.insert(id, handle);

        Ok(id)
    }

    fn read(&mut self, file: usize, buf: &mut [u8]) -> Result<usize> {
        let mut handle = self.handles.get_mut(&file).ok_or(Error::new(EBADF))?;

        if ! handle.is_connected() {
            Err(Error::new(ENOTCONN))
        } else if let Some((_ip, tcp)) = handle.data.pop_front() {
            let mut i = 0;
            while i < buf.len() && i < tcp.data.len() {
                buf[i] = tcp.data[i];
                i += 1;
            }

            Ok(i)
        } else if handle.flags & O_NONBLOCK == O_NONBLOCK || handle.read_closed() {
            Ok(0)
        } else {
            Err(Error::new(EWOULDBLOCK))
        }
    }

    fn write(&mut self, file: usize, buf: &[u8]) -> Result<usize> {
        let mut handle = self.handles.get_mut(&file).ok_or(Error::new(EBADF))?;

        if ! handle.is_connected() {
            Err(Error::new(ENOTCONN))
        } else if buf.len() >= 65507 {
            Err(Error::new(EMSGSIZE))
        } else {
            match handle.state {
                State::Established => {
                    let tcp = handle.create_tcp(TCP_ACK | TCP_PSH, buf.to_vec());
                    let ip = handle.create_ip(self.rng.gen(), tcp.to_bytes());
                    self.tcp_file.write(&ip.to_bytes()).map_err(|err| err.into_sys())?;
                    handle.seq += buf.len() as u32;
                    Ok(buf.len())
                },
                _ => {
                    Err(Error::new(EWOULDBLOCK))
                }
            }
        }
    }

    fn fevent(&mut self, file: usize, flags: usize) -> Result<usize> {
        let mut handle = self.handles.get_mut(&file).ok_or(Error::new(EBADF))?;

        handle.events = flags;

        Ok(file)
    }

    fn fpath(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let handle = self.handles.get(&id).ok_or(Error::new(EBADF))?;

        let path_string = format!("tcp:{}:{}/{}:{}", handle.remote.0.to_string(), handle.remote.1, handle.local.0.to_string(), handle.local.1);
        let path = path_string.as_bytes();

        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }

        Ok(i)
    }

    fn fsync(&mut self, file: usize) -> Result<usize> {
        let _handle = self.handles.get(&file).ok_or(Error::new(EBADF))?;

        Ok(0)
    }

    fn close(&mut self, file: usize) -> Result<usize> {

        let closed = {
            let mut handle = self.handles.get_mut(&file).ok_or(Error::new(EBADF))?;

            handle.data.clear();

            match handle.state {
                State::SynReceived | State::Established => {
                    handle.state = State::FinWait1;

                    let tcp = handle.create_tcp(TCP_FIN | TCP_ACK, Vec::new());
                    let ip = handle.create_ip(self.rng.gen(), tcp.to_bytes());
                    self.tcp_file.write(&ip.to_bytes()).map_err(|err| err.into_sys())?;

                    handle.seq += 1;

                    false
                },
                State::CloseWait => {
                    handle.state = State::LastAck;

                    let tcp = handle.create_tcp(TCP_FIN | TCP_ACK, Vec::new());
                    let ip = handle.create_ip(self.rng.gen(), tcp.to_bytes());
                    self.tcp_file.write(&ip.to_bytes()).map_err(|err| err.into_sys())?;

                    handle.seq += 1;

                    false
                },
                _ => true
            }
        };

        if closed {
            let handle = self.handles.remove(&file).ok_or(Error::new(EBADF))?;

            let remove = if let Some(mut port) = self.ports.get_mut(&handle.local.1) {
                *port = *port + 1;
                *port == 0
            } else {
                false
            };

            if remove {
                self.ports.remove(&handle.local.1);
            }
        }

        Ok(0)
    }
}

fn main() {
    thread::spawn(move || {
        let scheme_fd = syscall::open(":tcp", O_RDWR | O_CREAT | O_NONBLOCK).expect("tcpd: failed to create :tcp");
        let scheme_file = unsafe { File::from_raw_fd(scheme_fd) };

        let tcp_fd = syscall::open("ip:6", O_RDWR | O_NONBLOCK).expect("tcpd: failed to open ip:6");
        let tcp_file = unsafe { File::from_raw_fd(tcp_fd) };

        let tcpd = Rc::new(RefCell::new(Tcpd::new(scheme_file, tcp_file)));

        let mut event_queue = EventQueue::<()>::new().expect("tcpd: failed to create event queue");

        let tcp_tcpd = tcpd.clone();
        event_queue.add(tcp_fd, move |_count: usize| -> io::Result<Option<()>> {
            tcp_tcpd.borrow_mut().tcp_event()?;
            Ok(None)
        }).expect("tcpd: failed to listen to events on ip:6");

        event_queue.add(scheme_fd, move |_count: usize| -> io::Result<Option<()>> {
            tcpd.borrow_mut().scheme_event()?;
            Ok(None)
        }).expect("tcpd: failed to listen to events on :tcp");

        event_queue.trigger_all(0).expect("tcpd: failed to trigger event queue");

        event_queue.run().expect("tcpd: failed to run event queue");
    });
}
