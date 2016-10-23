extern crate event;
extern crate netutils;
extern crate syscall;

use event::EventQueue;
use std::cell::RefCell;
use std::fs::File;
use std::io::{Result, Read, Write};
use std::os::unix::io::FromRawFd;
use std::rc::Rc;
use std::thread;

use syscall::{Packet, SchemeMut, EWOULDBLOCK};

use scheme::EthernetScheme;

mod scheme;

fn main() {
    thread::spawn(move || {
        let network_fd = syscall::open("network:", syscall::O_RDWR | syscall::O_NONBLOCK).expect("ethernetd: failed to open network");
        let network = unsafe { File::from_raw_fd(network_fd) };

        let socket_fd = syscall::open(":ethernet", syscall::O_RDWR | syscall::O_CREAT | syscall::O_NONBLOCK).expect("ethernetd: failed to create ethernet scheme");
        let socket = Rc::new(RefCell::new(unsafe { File::from_raw_fd(socket_fd) }));

        let scheme = Rc::new(RefCell::new(EthernetScheme::new(network)));

        let todo = Rc::new(RefCell::new(Vec::<Packet>::new()));

        let mut event_queue = EventQueue::<()>::new().expect("ethernetd: failed to create event queue");

        let socket_net = socket.clone();
        let scheme_net = scheme.clone();
        let todo_net = todo.clone();
        event_queue.add(network_fd, move |_count: usize| -> Result<Option<()>> {
            if scheme_net.borrow_mut().input()? > 0 {
                let mut todo = todo_net.borrow_mut();
                let mut i = 0;
                while i < todo.len() {
                    let a = todo[i].a;
                    scheme_net.borrow_mut().handle(&mut todo[i]);
                    if todo[i].a == (-EWOULDBLOCK) as usize {
                        todo[i].a = a;
                        i += 1;
                    } else {
                        socket_net.borrow_mut().write(&mut todo[i])?;
                        todo.remove(i);
                    }
                }

                for (id, handle) in scheme_net.borrow_mut().handles.iter() {
                    if let Some(frame) = handle.frames.get(0) {
                        socket_net.borrow_mut().write(&Packet {
                            id: 0,
                            pid: 0,
                            uid: 0,
                            gid: 0,
                            a: syscall::number::SYS_FEVENT,
                            b: *id,
                            c: syscall::flag::EVENT_READ,
                            d: frame.data.len()
                        })?;
                    }
                }
            }
            Ok(None)
        }).expect("ethernetd: failed to listen for network events");

        event_queue.add(socket_fd, move |_count: usize| -> Result<Option<()>> {
            let mut packet = Packet::default();
            socket.borrow_mut().read(&mut packet)?;

            let a = packet.a;
            scheme.borrow_mut().handle(&mut packet);
            if packet.a == (-EWOULDBLOCK) as usize {
                packet.a = a;
                todo.borrow_mut().push(packet);
            } else {
                socket.borrow_mut().write(&mut packet)?;
            }

            Ok(None)
        }).expect("ethernetd: failed to listen for scheme events");

        event_queue.run().expect("ethernetd: failed to run event loop");
    });
}
