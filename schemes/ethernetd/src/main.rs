extern crate rand;
extern crate event;
extern crate netutils;
extern crate syscall;
extern crate smoltcp;

use event::EventQueue;
use std::cell::RefCell;
use std::fs::File;
use std::io::{Result, Read, Write};
use std::os::unix::io::FromRawFd;
use std::process;
use std::rc::Rc;

use syscall::{Packet, SchemeMut, EWOULDBLOCK};

use scheme::EthernetScheme;

mod device;
mod scheme;

fn daemon(network_fd: usize, ethernet_fd: usize) {
    let network_file = unsafe { File::from_raw_fd(network_fd) };
    let ethernet_file = Rc::new(RefCell::new(unsafe { File::from_raw_fd(ethernet_fd) }));
    let ethernet_scheme = Rc::new(RefCell::new(EthernetScheme::new(network_file)));
    let todo = Rc::new(RefCell::new(Vec::<Packet>::new()));

    let mut event_queue = EventQueue::<()>::new().expect("ethernetd: failed to create event queue");

    let ethernet_file_net = ethernet_file.clone();
    let ethernet_scheme_net = ethernet_scheme.clone();
    let todo_net = todo.clone();

    event_queue.add(network_fd, move |_count: usize| -> Result<Option<()>> {
        // FIXME: handle the errors properly.
        ethernet_scheme_net.borrow_mut().init("00:00:AA:BB:CC:DD");
        ethernet_scheme_net.borrow_mut().poll();

        // try to process pending operations on the ethernet scheme, if any.
        let mut todo = todo_net.borrow_mut();
        let mut i = 0;
        while i < todo.len() {
            let a = todo[i].a;
            ethernet_scheme_net.borrow_mut().handle(&mut todo[i]);
            if todo[i].a == (-EWOULDBLOCK) as usize {
                // this operation would block, we postpone it.
                todo[i].a = a;
                i += 1;
            } else {
                // the operation was processed: we write the outcome of the operation back to the
                // scheme file and remove it from the queue of pending operations
                ethernet_file_net.borrow_mut().write(&mut todo[i])?;
                todo.remove(i);
            }
        }
        Ok(None)
    }).expect("ethernetd: failed to listen for network events");

    event_queue.add(ethernet_fd, move |_count: usize| -> Result<Option<()>> {
        // keep processing operations until the scheme file is empty.
        loop {
            let mut packet = Packet::default();

            // we processed everything, time to return
            if ethernet_file.borrow_mut().read(&mut packet)? == 0 {
                break;
            }

            // `packet.a` is the event id, but if we can't process the event right away, it is
            // changed to `-EWOULDBLOCK`. In order to be able to process the event later if
            // necessary, we backup `packet.a` now, and we will restore it.
            let a = packet.a;

            // try to handle the event.
            ethernet_scheme.borrow_mut().handle(&mut packet);

            if packet.a == (-EWOULDBLOCK) as usize {
                // Can't process the event now. Restore the event id and save the event for later.
                packet.a = a;
                todo.borrow_mut().push(packet);
            } else {
                // The event was handled. We write the outcome in the scheme file.
                ethernet_file.borrow_mut().write(&mut packet)?;
                // FIXME: handle errors
                ethernet_scheme.borrow_mut().poll();
            }
        }


        Ok(None)
    }).expect("ethernetd: failed to listen for scheme events");

    event_queue.trigger_all(0).expect("ethernetd: failed to trigger events");

    event_queue.run().expect("ethernetd: failed to run event loop");
}

fn main() {
    match syscall::open("network:", syscall::O_RDWR | syscall::O_NONBLOCK) {
        Ok(network_fd) => {
            // Daemonize
            if unsafe { syscall::clone(0).unwrap() } == 0 {
                match syscall::open(":ethernet", syscall::O_RDWR | syscall::O_CREAT | syscall::O_NONBLOCK) {
                    Ok(ethernet_fd) => {
                        daemon(network_fd, ethernet_fd);
                    },
                    Err(err) => {
                        println!("ethernetd: failed to create ethernet scheme: {}", err);
                        process::exit(1);
                    }
                }
            }
        },
        Err(err) => {
            println!("ethernetd: failed to open network: {}", err);
            process::exit(1);
        }
    }
}
