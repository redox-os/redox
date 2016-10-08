#![feature(asm)]

extern crate dma;
extern crate syscall;

use std::{env, thread};
use std::fs::File;
use std::io::{Read, Write};

use syscall::{iopl, physmap, physunmap, Packet, MAP_WRITE};

fn main() {
    let mut args = env::args().skip(1);

    let bar_str = args.next().expect("e1000d: no address provided");
    let bar = usize::from_str_radix(&bar_str, 16).expect("e1000d: failed to parse address");

    let irq_str = args.next().expect("e1000d: no irq provided");
    let irq = irq_str.parse::<u8>().expect("e1000d: failed to parse irq");

    thread::spawn(move || {
        unsafe {
            iopl(3).expect("e1000d: failed to get I/O permission");
            asm!("cli" :::: "intel", "volatile");
        }

        let address = unsafe { physmap(bar, 4096, MAP_WRITE).expect("e1000d: failed to map address") };
        {
            println!("e1000d {:X}", bar);
            let mut socket = File::create(":network").expect("e1000d: failed to create network scheme");
            //let scheme = DiskScheme::new(ahci::disks(address, irq));
            loop {
                let mut packet = Packet::default();
                socket.read(&mut packet).expect("e1000d: failed to read network scheme");
                //scheme.handle(&mut packet);
                socket.write(&mut packet).expect("e1000d: failed to read network scheme");
            }
        }
        unsafe { let _ = physunmap(address); }
    });
}
