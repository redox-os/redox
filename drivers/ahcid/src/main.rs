#![feature(asm)]
#![feature(question_mark)]

#[macro_use]
extern crate bitflags;
extern crate io;
extern crate syscall;

use std::{env, thread, usize};

use syscall::{iopl, physmap, physunmap, MAP_WRITE};

pub mod ahci;

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
            let mut disks = ahci::disks(address, irq);
            for mut disk in disks.iter_mut() {
                let mut sector = [0; 512];
                println!("Read disk {} size {} MB", disk.id(), disk.size()/1024/1024);
                match disk.read(0, &mut sector) {
                    Ok(count) => {
                        println!("{}", count);
                        for i in 0..512 {
                            print!("{:X} ", sector[i]);
                        }
                        println!("");
                    },
                    Err(err) => {
                        println!("{}", err);
                    }
                }
            }
            loop {
                let _ = syscall::sched_yield();
            }
        }
        unsafe { let _ = physunmap(address); }
    });
}
