#![feature(asm)]
#![feature(question_mark)]

#[macro_use]
extern crate bitflags;
extern crate dma;
extern crate io;
extern crate spin;
extern crate syscall;

use std::fs::File;
use std::io::{Read, Write};
use std::{env, thread, usize};
use syscall::{iopl, physmap, physunmap, MAP_WRITE, Packet, Scheme};

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
            iopl(3).expect("ahcid: failed to get I/O permission");
            asm!("cli" :::: "intel", "volatile");
        }

        let address = unsafe { physmap(bar, 4096, MAP_WRITE).expect("ahcid: failed to map address") };
        {
            let mut socket = File::create(":disk").expect("ahcid: failed to create disk scheme");
            let scheme = DiskScheme::new(ahci::disks(address, irq));
            loop {
                let mut packet = Packet::default();
                socket.read(&mut packet).expect("ahcid: failed to read disk scheme");
                scheme.handle(&mut packet);
                socket.write(&mut packet).expect("ahcid: failed to read disk scheme");
            }
        }
        unsafe { let _ = physunmap(address); }
    });
}
