extern crate event;
extern crate netutils;
extern crate rand;
extern crate syscall;

use rand::{Rng, OsRng};
use std::collections::{BTreeMap, VecDeque};
use std::cell::RefCell;
use std::fs::File;
use std::io::{self, Read, Write};
use std::{mem, slice, str};
use std::ops::{Deref, DerefMut};
use std::os::unix::io::FromRawFd;
use std::rc::Rc;

use event::EventQueue;
use netutils::{n16, Ipv4, Ipv4Addr, Ipv4Header, Checksum};
use netutils::udp::{Udp, UdpHeader};
use syscall::data::{Packet, TimeSpec};
use syscall::error::{Error, Result, EACCES, EADDRINUSE, EBADF, EIO, EINVAL, EMSGSIZE, ENOTCONN, EWOULDBLOCK};
use syscall::flag::{EVENT_READ, F_GETFL, F_SETFL, O_ACCMODE, O_CREAT, O_RDWR, O_NONBLOCK};
use syscall::scheme::SchemeMut;

fn parse_socket(socket: &str) -> (Ipv4Addr, u16) {
    let mut socket_parts = socket.split(":");
    let host = Ipv4Addr::from_str(socket_parts.next().unwrap_or(""));
    let port = socket_parts.next().unwrap_or("").parse::<u16>().unwrap_or(0);
    (host, port)
}

struct UdpHandle {
    local: (Ipv4Addr, u16),
    remote: (Ipv4Addr, u16),
    flags: usize,
    events: usize,
    read_timeout: Option<TimeSpec>,
    write_timeout: Option<TimeSpec>,
    ttl: u8,
    data: VecDeque<Vec<u8>>,
    todo: VecDeque<Packet>,
}

#[derive(Copy, Clone)]
enum SettingKind {
    Read,
    Write
}

enum Handle {
    Udp(UdpHandle),
    Setting(usize, SettingKind),
}

struct Udpd {
    scheme_file: File,
    udp_file: File,
    ports: BTreeMap<u16, usize>,
    next_id: usize,
    handles: BTreeMap<usize, Handle>,
    rng: OsRng,
}

impl Udpd {
    fn new(scheme_file: File, udp_file: File) -> Self {
        Udpd {
            scheme_file: scheme_file,
            udp_file: udp_file,
            ports: BTreeMap::new(),
            next_id: 1,
            handles: BTreeMap::new(),
            rng: OsRng::new().expect("udpd: failed to open RNG")
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
                packet.a = a;
                if let Some(mut handle) = self.handles.get_mut(&packet.b) {
                    if let Handle::Udp(ref mut handle) = *handle {
                        handle.todo.push_back(packet);
                    }
                }
            } else {
                self.scheme_file.write(&packet)?;
            }
        }

        Ok(())
    }

    fn udp_event(&mut self) -> io::Result<()> {
        loop {
            let mut bytes = [0; 65536];
            let count = self.udp_file.read(&mut bytes)?;
            if count == 0 {
                break;
            }
            if let Some(ip) = Ipv4::from_bytes(&bytes[.. count]) {
                if let Some(udp) = Udp::from_bytes(&ip.data) {
                    for (id, handle) in self.handles.iter_mut() {
                        if let Handle::Udp(ref mut handle) = *handle {
                                // Local address not set or IP dst matches or is broadcast
                            if (handle.local.0 == Ipv4Addr::NULL || ip.header.dst == handle.local.0 || ip.header.dst == Ipv4Addr::BROADCAST)
                                // Local port matches UDP dst
                                && udp.header.dst.get() == handle.local.1
                                // Remote address not set or is broadcast, or IP src matches
                                && (handle.remote.0 == Ipv4Addr::NULL || handle.remote.0 == Ipv4Addr::BROADCAST || ip.header.src == handle.remote.0)
                                // Remote port not set or UDP src matches
                                && (handle.remote.1 == 0 || udp.header.src.get() == handle.remote.1)
                            {
                                handle.data.push_back(udp.data.clone());

                                while ! handle.todo.is_empty() && ! handle.data.is_empty() {
                                    let mut packet = handle.todo.pop_front().unwrap();
                                    let buf = unsafe { slice::from_raw_parts_mut(packet.c as *mut u8, packet.d) };
                                    let data = handle.data.pop_front().unwrap();

                                    let mut i = 0;
                                    while i < buf.len() && i < data.len() {
                                        buf[i] = data[i];
                                        i += 1;
                                    }
                                    packet.a = i;

                                    self.scheme_file.write(&packet)?;
                                }

                                if handle.events & EVENT_READ == EVENT_READ {
                                    if let Some(data) = handle.data.get(0) {
                                        self.scheme_file.write(&Packet {
                                            id: 0,
                                            pid: 0,
                                            uid: 0,
                                            gid: 0,
                                            a: syscall::number::SYS_FEVENT,
                                            b: *id,
                                            c: EVENT_READ,
                                            d: data.len()
                                        })?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl SchemeMut for Udpd {
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

        self.ports.insert(local.1, 1);

        let id = self.next_id;
        self.next_id += 1;

        self.handles.insert(id, Handle::Udp(UdpHandle {
            local: local,
            remote: remote,
            flags: flags,
            events: 0,
            ttl: 64,
            read_timeout: None,
            write_timeout: None,
            data: VecDeque::new(),
            todo: VecDeque::new(),
        }));

        Ok(id)
    }

    fn dup(&mut self, file: usize, buf: &[u8]) -> Result<usize> {
        let handle = match *self.handles.get(&file).ok_or(Error::new(EBADF))? {
            Handle::Udp(ref handle) => {
                let mut handle = UdpHandle {
                    local: handle.local,
                    remote: handle.remote,
                    flags: handle.flags,
                    events: 0,
                    ttl: handle.ttl,
                    read_timeout: handle.read_timeout,
                    write_timeout: handle.write_timeout,
                    data: handle.data.clone(),
                    todo: VecDeque::new(),
                };

                let path = str::from_utf8(buf).or(Err(Error::new(EINVAL)))?;

                if path == "read_timeout" {
                    Handle::Setting(file, SettingKind::Read)
                } else if path == "write_timeout" {
                    Handle::Setting(file, SettingKind::Write)
                } else {
                    if handle.remote.0 == Ipv4Addr::NULL || handle.remote.1 == 0 {
                        handle.remote = parse_socket(path);
                    }

                    if let Some(mut port) = self.ports.get_mut(&handle.local.1) {
                        *port = *port + 1;
                    }

                    Handle::Udp(handle)
                }
            },
            Handle::Setting(file, kind) => {
                Handle::Setting(file, kind)
            }
        };

        let id = self.next_id;
        self.next_id += 1;

        self.handles.insert(id, handle);

        Ok(id)
    }

    fn read(&mut self, file: usize, buf: &mut [u8]) -> Result<usize> {
        let (file, kind) = match *self.handles.get_mut(&file).ok_or(Error::new(EBADF))? {
            Handle::Udp(ref mut handle) => {
                if handle.remote.0 == Ipv4Addr::NULL || handle.remote.1 == 0 {
                    return Err(Error::new(ENOTCONN));
                } else if let Some(data) = handle.data.pop_front() {
                    let mut i = 0;
                    while i < buf.len() && i < data.len() {
                        buf[i] = data[i];
                        i += 1;
                    }

                    return Ok(i);
                } else if handle.flags & O_NONBLOCK == O_NONBLOCK {
                    return Ok(0);
                } else {
                    return Err(Error::new(EWOULDBLOCK));
                }
            },
            Handle::Setting(file, kind) => {
                (file, kind)
            }
        };

        if let Handle::Udp(ref mut handle) = *self.handles.get_mut(&file).ok_or(Error::new(EBADF))? {
            let timeout = match kind {
                SettingKind::Read => handle.read_timeout,
                SettingKind::Write => handle.write_timeout
            };

            let count = if let Some(timespec) = timeout {
                timespec.deref().read(buf).map_err(|err| Error::new(err.raw_os_error().unwrap_or(EIO)))?
            } else {
                0
            };

            Ok(count)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn write(&mut self, file: usize, buf: &[u8]) -> Result<usize> {
        let (file, kind) = match *self.handles.get(&file).ok_or(Error::new(EBADF))? {
            Handle::Udp(ref handle) => {
                if handle.remote.0 == Ipv4Addr::NULL || handle.remote.1 == 0 {
                    return Err(Error::new(ENOTCONN));
                } else if buf.len() >= 65507 {
                    return Err(Error::new(EMSGSIZE));
                } else {
                    let udp_data = buf.to_vec();

                    let udp = Udp {
                        header: UdpHeader {
                            src: n16::new(handle.local.1),
                            dst: n16::new(handle.remote.1),
                            len: n16::new((udp_data.len() + mem::size_of::<UdpHeader>()) as u16),
                            checksum: Checksum { data: 0 }
                        },
                        data: udp_data
                    };

                    let ip_data = udp.to_bytes();

                    let ip = Ipv4 {
                        header: Ipv4Header {
                            ver_hlen: 0x45,
                            services: 0,
                            len: n16::new((ip_data.len() + mem::size_of::<Ipv4Header>()) as u16),
                            id: n16::new(self.rng.gen()),
                            flags_fragment: n16::new(0),
                            ttl: handle.ttl,
                            proto: 0x11,
                            checksum: Checksum { data: 0 },
                            src: handle.local.0,
                            dst: handle.remote.0
                        },
                        options: Vec::new(),
                        data: ip_data
                    };

                    return self.udp_file.write(&ip.to_bytes()).map_err(|err| Error::new(err.raw_os_error().unwrap_or(EIO))).and(Ok(buf.len()));
                }
            },
            Handle::Setting(file, kind) => {
                (file, kind)
            }
        };

        if let Handle::Udp(ref mut handle) = *self.handles.get_mut(&file).ok_or(Error::new(EBADF))? {
            let (count, timeout) = if buf.len() >= mem::size_of::<TimeSpec>() {
                let mut timespec = TimeSpec::default();
                let count = timespec.deref_mut().write(buf).map_err(|err| Error::new(err.raw_os_error().unwrap_or(EIO)))?;
                (count, Some(timespec))
            } else {
                (0, None)
            };

            match kind {
                SettingKind::Read => handle.read_timeout = timeout,
                SettingKind::Write => handle.write_timeout = timeout
            }

            Ok(count)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn fcntl(&mut self, file: usize, cmd: usize, arg: usize) -> Result<usize> {
        if let Handle::Udp(ref mut handle) = *self.handles.get_mut(&file).ok_or(Error::new(EBADF))? {
            match cmd {
                F_GETFL => Ok(handle.flags),
                F_SETFL => {
                    handle.flags = arg & ! O_ACCMODE;
                    Ok(0)
                },
                _ => Err(Error::new(EINVAL))
            }
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn fevent(&mut self, file: usize, flags: usize) -> Result<usize> {
        if let Handle::Udp(ref mut handle) = *self.handles.get_mut(&file).ok_or(Error::new(EBADF))? {
            handle.events = flags;
            Ok(file)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn fpath(&mut self, file: usize, buf: &mut [u8]) -> Result<usize> {
        if let Handle::Udp(ref mut handle) = *self.handles.get_mut(&file).ok_or(Error::new(EBADF))? {
            let path_string = format!("udp:{}:{}/{}:{}", handle.remote.0.to_string(), handle.remote.1, handle.local.0.to_string(), handle.local.1);
            let path = path_string.as_bytes();

            let mut i = 0;
            while i < buf.len() && i < path.len() {
                buf[i] = path[i];
                i += 1;
            }

            Ok(i)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn fsync(&mut self, file: usize) -> Result<usize> {
        let _handle = self.handles.get(&file).ok_or(Error::new(EBADF))?;

        Ok(0)
    }

    fn close(&mut self, file: usize) -> Result<usize> {
        let handle = self.handles.remove(&file).ok_or(Error::new(EBADF))?;

        if let Handle::Udp(ref handle) = handle {
            let remove = if let Some(mut port) = self.ports.get_mut(&handle.local.1) {
                *port = *port + 1;
                *port == 0
            } else {
                false
            };

            if remove {
                drop(self.ports.remove(&handle.local.1));
            }
        }

        drop(handle);

        Ok(0)
    }
}

fn main() {
    // Daemonize
    if unsafe { syscall::clone(0).unwrap() } == 0 {
        let scheme_fd = syscall::open(":udp", O_RDWR | O_CREAT | O_NONBLOCK).expect("udpd: failed to create :udp");
        let scheme_file = unsafe { File::from_raw_fd(scheme_fd) };

        let udp_fd = syscall::open("ip:11", O_RDWR | O_NONBLOCK).expect("udpd: failed to open ip:11");
        let udp_file = unsafe { File::from_raw_fd(udp_fd) };

        let udpd = Rc::new(RefCell::new(Udpd::new(scheme_file, udp_file)));

        let mut event_queue = EventQueue::<()>::new().expect("udpd: failed to create event queue");

        let udp_udpd = udpd.clone();
        event_queue.add(udp_fd, move |_count: usize| -> io::Result<Option<()>> {
            udp_udpd.borrow_mut().udp_event()?;
            Ok(None)
        }).expect("udpd: failed to listen to events on ip:11");

        event_queue.add(scheme_fd, move |_count: usize| -> io::Result<Option<()>> {
            udpd.borrow_mut().scheme_event()?;
            Ok(None)
        }).expect("udpd: failed to listen to events on :udp");

        event_queue.trigger_all(0).expect("udpd: failed to trigger event queue");

        event_queue.run().expect("udpd: failed to run event queue");
    }
}
