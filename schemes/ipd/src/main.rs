extern crate event;
extern crate netutils;
extern crate syscall;

use event::EventQueue;
use netutils::{Ipv4Addr, Ipv4, Tcp};
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::unix::io::FromRawFd;
use std::{process, slice, str};
use std::rc::Rc;
use syscall::data::Packet;
use syscall::error::{Error, Result, EACCES, EADDRNOTAVAIL, EBADF, EIO, EINVAL, ENOENT, EWOULDBLOCK};
use syscall::flag::{EVENT_READ, O_NONBLOCK};
use syscall::scheme::SchemeMut;

use interface::{Interface, EthernetInterface, LoopbackInterface};

mod interface;

struct Handle {
    proto: u8,
    flags: usize,
    events: usize,
    data: VecDeque<Vec<u8>>,
    todo: VecDeque<Packet>,
}

struct Ipd {
    scheme_file: File,
    interfaces: Vec<Box<Interface>>,
    next_id: usize,
    handles: BTreeMap<usize, Handle>,
}

impl Ipd {
    fn new(scheme_file: File) -> Self {
        Ipd {
            scheme_file: scheme_file,
            interfaces: Vec::new(),
            next_id: 1,
            handles: BTreeMap::new(),
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
                    handle.todo.push_back(packet);
                }
            } else {
                self.scheme_file.write(&packet)?;
            }
        }

        Ok(())
    }

    fn ip_event(&mut self, if_id: usize) -> io::Result<()> {
        if let Some(mut interface) = self.interfaces.get_mut(if_id) {
            for ip in interface.recv()? {
                for (id, handle) in self.handles.iter_mut() {
                    if ip.header.proto == handle.proto {
                        handle.data.push_back(ip.to_bytes());

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

        Ok(())
    }

    fn loopback_event(&mut self, loopback_id: usize) -> io::Result<()> {
        let handle_loopback = if let Some(interface) = self.interfaces.get(loopback_id) {
            interface.has_loopback_data()
        } else {
            false
        };

        if handle_loopback {
            self.ip_event(loopback_id)?;
        }

        Ok(())
    }
}

impl SchemeMut for Ipd {
    fn open(&mut self, url: &[u8], flags: usize, uid: u32, _gid: u32) -> Result<usize> {
        if uid == 0 {
            let path = str::from_utf8(url).or(Err(Error::new(EINVAL)))?;

            let proto = u8::from_str_radix(path, 16).or(Err(Error::new(ENOENT)))?;

            let id = self.next_id;
            self.next_id += 1;

            self.handles.insert(id, Handle {
                proto: proto,
                flags: flags,
                events: 0,
                data: VecDeque::new(),
                todo: VecDeque::new(),
            });

            Ok(id)
        } else {
            Err(Error::new(EACCES))
        }
    }

    fn dup(&mut self, file: usize, _buf: &[u8]) -> Result<usize> {
        let handle = {
            let handle = self.handles.get(&file).ok_or(Error::new(EBADF))?;
            Handle {
                proto: handle.proto,
                flags: handle.flags,
                events: 0,
                data: handle.data.clone(),
                todo: VecDeque::new(),
            }
        };

        let id = self.next_id;
        self.next_id += 1;

        self.handles.insert(id, handle);

        Ok(id)
    }

    fn read(&mut self, file: usize, buf: &mut [u8]) -> Result<usize> {
        let mut handle = self.handles.get_mut(&file).ok_or(Error::new(EBADF))?;

        if let Some(data) = handle.data.pop_front() {
            let mut i = 0;
            while i < buf.len() && i < data.len() {
                buf[i] = data[i];
                i += 1;
            }

            Ok(i)
        } else if handle.flags & O_NONBLOCK == O_NONBLOCK {
            Ok(0)
        } else {
            Err(Error::new(EWOULDBLOCK))
        }
    }

    fn write(&mut self, file: usize, buf: &[u8]) -> Result<usize> {
        let handle = self.handles.get(&file).ok_or(Error::new(EBADF))?;

        if let Some(mut ip) = Ipv4::from_bytes(buf) {
            for mut interface in self.interfaces.iter_mut() {
                let if_ip = interface.ip();
                if ip.header.src == if_ip || (ip.header.src == Ipv4Addr::NULL && interface.routable(ip.header.dst)) {
                    ip.header.src = if_ip;
                    ip.header.proto = handle.proto;

                    if let Some(mut tcp) = Tcp::from_bytes(&ip.data) {
                        tcp.checksum(&ip.header.src, &ip.header.dst);
                        ip.data = tcp.to_bytes();
                    }

                    ip.checksum();

                    interface.send(ip).map_err(|err| Error::new(err.raw_os_error().unwrap_or(EIO)))?;

                    return Ok(buf.len());
                }
            }

            Err(Error::new(EADDRNOTAVAIL))
        } else {
            Err(Error::new(EINVAL))
        }
    }

    fn fevent(&mut self, file: usize, flags: usize) -> Result<usize> {
        let mut handle = self.handles.get_mut(&file).ok_or(Error::new(EBADF))?;

        handle.events = flags;

        Ok(file)
    }

    fn fpath(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let handle = self.handles.get(&id).ok_or(Error::new(EBADF))?;

        let path_string = format!("ip:{:X}", handle.proto);
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
        let handle = self.handles.remove(&file).ok_or(Error::new(EBADF))?;

        drop(handle);

        Ok(0)
    }
}

fn daemon(arp_fd: usize, ip_fd: usize, scheme_fd: usize) {
    let scheme_file = unsafe { File::from_raw_fd(scheme_fd) };

    let ipd = Rc::new(RefCell::new(Ipd::new(scheme_file)));

    let mut event_queue = EventQueue::<()>::new().expect("ipd: failed to create event queue");

    //TODO: Multiple interfaces
    {
        let if_id = {
            let mut ipd = ipd.borrow_mut();
            let if_id = ipd.interfaces.len();
            ipd.interfaces.push(Box::new(EthernetInterface::new(arp_fd, ip_fd)));
            if_id
        };

        let arp_ipd = ipd.clone();
        event_queue.add(arp_fd, move |_count: usize| -> io::Result<Option<()>> {
            if let Some(mut interface) = arp_ipd.borrow_mut().interfaces.get_mut(if_id) {
                interface.arp_event()?;
            }

            Ok(None)
        }).expect("ipd: failed to listen to events on ethernet:806");

        let ip_ipd = ipd.clone();
        event_queue.add(ip_fd, move |_count: usize| -> io::Result<Option<()>> {
            ip_ipd.borrow_mut().ip_event(if_id)?;

            Ok(None)
        }).expect("ipd: failed to listen to events on ethernet:800");
    }

    let loopback_id = {
        let mut ipd = ipd.borrow_mut();
        let if_id = ipd.interfaces.len();
        ipd.interfaces.push(Box::new(LoopbackInterface::new()));
        if_id
    };

    event_queue.add(scheme_fd, move |_count: usize| -> io::Result<Option<()>> {
        let mut ipd = ipd.borrow_mut();

        ipd.loopback_event(loopback_id)?;
        ipd.scheme_event()?;
        ipd.loopback_event(loopback_id)?;

        Ok(None)
    }).expect("ipd: failed to listen to events on :ip");

    // Make sure that all descriptors are at EOF
    event_queue.trigger_all(0).expect("ipd: failed to trigger event queue");

    event_queue.run().expect("ipd: failed to run event queue");
}

fn main() {
    match syscall::open("ethernet:806", syscall::O_RDWR | syscall::O_NONBLOCK) {
        Ok(arp_fd) => match syscall::open("ethernet:800", syscall::O_RDWR | syscall::O_NONBLOCK) {
            Ok(ip_fd) => {
                // Daemonize
                if unsafe { syscall::clone(0).unwrap() } == 0 {
                    match syscall::open(":ip", syscall::O_RDWR | syscall::O_CREAT | syscall::O_NONBLOCK) {
                        Ok(scheme_fd) => {
                            daemon(arp_fd, ip_fd, scheme_fd);
                        },
                        Err(err) => {
                            println!("ipd: failed to create ip scheme: {}", err);
                            process::exit(1);
                        }
                    }
                }
            },
            Err(err) => {
                println!("ipd: failed to open ethernet:800: {}", err);
                process::exit(1);
            }
        },
        Err(err) => {
            println!("ipd: failed to open ethernet:806: {}", err);
            process::exit(1);
        }
    }
}
