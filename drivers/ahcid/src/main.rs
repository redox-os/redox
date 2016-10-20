#![feature(asm)]

#[macro_use]
extern crate bitflags;
extern crate dma;
extern crate io;
extern crate spin;
extern crate syscall;

use std::{env, thread, usize};
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};
use syscall::{EVENT_READ, MAP_WRITE, Event, Packet, Scheme};

use scheme::DiskScheme;

pub mod ahci;
pub mod scheme;

fn main() {
    let mut args = env::args().skip(1);

    let bar_str = args.next().expect("ahcid: no address provided");
    let bar = usize::from_str_radix(&bar_str, 16).expect("ahcid: failed to parse address");

    let irq_str = args.next().expect("ahcid: no irq provided");
    let irq = irq_str.parse::<u8>().expect("ahcid: failed to parse irq");

    thread::spawn(move || {
        unsafe {
            syscall::iopl(3).expect("ahcid: failed to get I/O permission");
            asm!("cli" :::: "intel", "volatile");
        }

        let address = unsafe { syscall::physmap(bar, 4096, MAP_WRITE).expect("ahcid: failed to map address") };
        {
            let socket_fd = syscall::open(":disk", syscall::O_RDWR | syscall::O_CREAT | syscall::O_NONBLOCK).expect("ahcid: failed to create disk scheme");
            let mut socket = unsafe { File::from_raw_fd(socket_fd) };
            syscall::fevent(socket_fd, EVENT_READ).expect("ahcid: failed to fevent disk scheme");

            let mut irq_file = File::open(&format!("irq:{}", irq)).expect("ahcid: failed to open irq file");
            let irq_fd = irq_file.as_raw_fd();
            syscall::fevent(irq_fd, EVENT_READ).expect("ahcid: failed to fevent irq file");

            let mut event_file = File::open("event:").expect("ahcid: failed to open event file");

            let scheme = DiskScheme::new(ahci::disks(address, irq));
            loop {
                let mut event = Event::default();
                event_file.read(&mut event).expect("ahcid: failed to read event file");
                if event.id == socket_fd {
                    let mut packet = Packet::default();
                    socket.read(&mut packet).expect("ahcid: failed to read disk scheme");
                    scheme.handle(&mut packet);
                    socket.write(&mut packet).expect("ahcid: failed to write disk scheme");
                } else if event.id == irq_fd {
                    let mut irq = [0; 8];
                    if irq_file.read(&mut irq).expect("ahcid: failed to read irq file") >= irq.len() {
                        //TODO : Test for IRQ
                        //irq_file.write(&irq).expect("ahcid: failed to write irq file");
                    }
                } else {
                    println!("Unknown event {}", event.id);
                }
            }
        }
        unsafe { let _ = syscall::physunmap(address); }
    });
}
