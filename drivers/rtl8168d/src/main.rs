#![feature(asm)]

extern crate dma;
extern crate event;
extern crate io;
extern crate netutils;
extern crate syscall;

use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::{Read, Write, Result};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::sync::Arc;

use event::EventQueue;
use syscall::{Packet, SchemeMut, MAP_WRITE};
use syscall::error::EWOULDBLOCK;

pub mod device;

fn main() {
    let mut args = env::args().skip(1);

    let bar_str = args.next().expect("rtl8168d: no address provided");
    let bar = usize::from_str_radix(&bar_str, 16).expect("rtl8168d: failed to parse address");

    let irq_str = args.next().expect("rtl8168d: no irq provided");
    let irq = irq_str.parse::<u8>().expect("rtl8168d: failed to parse irq");

    print!("{}", format!(" + RTL8168 on: {:X}, IRQ: {}\n", bar, irq));

    // Daemonize
    if unsafe { syscall::clone(0).unwrap() } == 0 {
        let socket_fd = syscall::open(":network", syscall::O_RDWR | syscall::O_CREAT | syscall::O_NONBLOCK).expect("rtl8168d: failed to create network scheme");
        let socket = Arc::new(RefCell::new(unsafe { File::from_raw_fd(socket_fd) }));

        let mut irq_file = File::open(format!("irq:{}", irq)).expect("rtl8168d: failed to open IRQ file");

        let address = unsafe { syscall::physmap(bar, 256, MAP_WRITE).expect("rtl8168d: failed to map address") };
        {
            let device = Arc::new(RefCell::new(unsafe { device::Rtl8168::new(address).expect("rtl8168d: failed to allocate device") }));

            let mut event_queue = EventQueue::<usize>::new().expect("rtl8168d: failed to create event queue");

            let todo = Arc::new(RefCell::new(Vec::<Packet>::new()));

            let device_irq = device.clone();
            let socket_irq = socket.clone();
            let todo_irq = todo.clone();
            event_queue.add(irq_file.as_raw_fd(), move |_count: usize| -> Result<Option<usize>> {
                let mut irq = [0; 8];
                irq_file.read(&mut irq)?;

                let isr = unsafe { device_irq.borrow_mut().irq() };
                if isr != 0 {
                    irq_file.write(&mut irq)?;

                    let mut todo = todo_irq.borrow_mut();
                    let mut i = 0;
                    while i < todo.len() {
                        let a = todo[i].a;
                        device_irq.borrow_mut().handle(&mut todo[i]);
                        if todo[i].a == (-EWOULDBLOCK) as usize {
                            todo[i].a = a;
                            i += 1;
                        } else {
                            socket_irq.borrow_mut().write(&mut todo[i])?;
                            todo.remove(i);
                        }
                    }

                    let next_read = device_irq.borrow().next_read();
                    if next_read > 0 {
                        return Ok(Some(next_read));
                    }
                }
                Ok(None)
            }).expect("rtl8168d: failed to catch events on IRQ file");

            let socket_fd = socket.borrow().as_raw_fd();
            let socket_packet = socket.clone();
            event_queue.add(socket_fd, move |_count: usize| -> Result<Option<usize>> {
                loop {
                    let mut packet = Packet::default();
                    if socket_packet.borrow_mut().read(&mut packet)? == 0 {
                        break;
                    }

                    let a = packet.a;
                    device.borrow_mut().handle(&mut packet);
                    if packet.a == (-EWOULDBLOCK) as usize {
                        packet.a = a;
                        todo.borrow_mut().push(packet);
                    } else {
                        socket_packet.borrow_mut().write(&mut packet)?;
                    }
                }

                let next_read = device.borrow().next_read();
                if next_read > 0 {
                    return Ok(Some(next_read));
                }

                Ok(None)
            }).expect("rtl8168d: failed to catch events on IRQ file");

            for event_count in event_queue.trigger_all(0).expect("rtl8168d: failed to trigger events") {
                socket.borrow_mut().write(&Packet {
                    id: 0,
                    pid: 0,
                    uid: 0,
                    gid: 0,
                    a: syscall::number::SYS_FEVENT,
                    b: 0,
                    c: syscall::flag::EVENT_READ,
                    d: event_count
                }).expect("rtl8168d: failed to write event");
            }

            loop {
                let event_count = event_queue.run().expect("rtl8168d: failed to handle events");

                socket.borrow_mut().write(&Packet {
                    id: 0,
                    pid: 0,
                    uid: 0,
                    gid: 0,
                    a: syscall::number::SYS_FEVENT,
                    b: 0,
                    c: syscall::flag::EVENT_READ,
                    d: event_count
                }).expect("rtl8168d: failed to write event");
            }
        }
        unsafe { let _ = syscall::physunmap(address); }
    }
}
