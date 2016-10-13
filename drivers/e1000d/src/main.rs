#![feature(asm)]
#![feature(question_mark)]

extern crate dma;
extern crate event;
extern crate syscall;

use std::cell::RefCell;
use std::{env, thread};
use std::fs::File;
use std::io::{Read, Write, Result};
use std::os::unix::io::AsRawFd;
use std::sync::Arc;

use event::EventQueue;
use syscall::{Packet, Scheme, MAP_WRITE};
use syscall::error::EWOULDBLOCK;

pub mod device;

fn main() {
    let mut args = env::args().skip(1);

    let bar_str = args.next().expect("e1000d: no address provided");
    let bar = usize::from_str_radix(&bar_str, 16).expect("e1000d: failed to parse address");

    let irq_str = args.next().expect("e1000d: no irq provided");
    let irq = irq_str.parse::<u8>().expect("e1000d: failed to parse irq");

    thread::spawn(move || {
        unsafe {
            syscall::iopl(3).expect("e1000d: failed to get I/O permission");
            asm!("cli" :::: "intel", "volatile");
        }

        let socket = Arc::new(RefCell::new(File::create(":network").expect("e1000d: failed to create network scheme")));

        let address = unsafe { syscall::physmap(bar, 128*1024, MAP_WRITE).expect("e1000d: failed to map address") };
        {
            let device = Arc::new(unsafe { device::Intel8254x::new(address, irq).expect("e1000d: failed to allocate device") });

            let mut event_queue = EventQueue::<()>::new().expect("e1000d: failed to create event queue");

            let todo = Arc::new(RefCell::new(Vec::<Packet>::new()));

            let device_irq = device.clone();
            let socket_irq = socket.clone();
            let todo_irq = todo.clone();
            let mut irq_file = File::open(format!("irq:{}", irq)).expect("e1000d: failed to open IRQ file");
            event_queue.add(irq_file.as_raw_fd(), move |_count: usize| -> Result<Option<()>> {
                let mut irq = [0; 8];
                irq_file.read(&mut irq)?;
                if unsafe { device_irq.irq() } {
                    irq_file.write(&mut irq)?;

                    let mut todo = todo_irq.borrow_mut();
                    let mut i = 0;
                    while i < todo.len() {
                        let a = todo[i].a;
                        device_irq.handle(&mut todo[i]);
                        if todo[i].a == (-EWOULDBLOCK) as usize {
                            todo[i].a = a;
                            i += 1;
                        } else {
                            socket_irq.borrow_mut().write(&mut todo[i])?;
                            todo.remove(i);
                        }
                    }
                }
                Ok(None)
            }).expect("e1000d: failed to catch events on IRQ file");

            let socket_fd = socket.borrow().as_raw_fd();
            event_queue.add(socket_fd, move |_count: usize| -> Result<Option<()>> {
                let mut packet = Packet::default();
                socket.borrow_mut().read(&mut packet)?;

                let a = packet.a;
                device.handle(&mut packet);
                if packet.a == (-EWOULDBLOCK) as usize {
                    packet.a = a;
                    todo.borrow_mut().push(packet);
                } else {
                    socket.borrow_mut().write(&mut packet)?;
                }

                Ok(None)
            }).expect("e1000d: failed to catch events on IRQ file");

            event_queue.run().expect("e1000d: failed to handle events");
        }
        unsafe { let _ = syscall::physunmap(address); }
    });
}
