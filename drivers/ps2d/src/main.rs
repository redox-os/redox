#![feature(asm)]

extern crate syscall;

use std::fs::File;
use std::io::{Read, Write};
use std::thread;

use syscall::iopl;

fn main() {
    if true {
        unsafe {
            iopl(3).expect("pskbd: failed to get I/O permission");
            asm!("cli" :::: "intel", "volatile");
        }

        let mut file = File::open("irq:1").expect("pskbd: failed to open irq:1");

        println!("pskbd: Reading keyboard IRQs");

        loop {
            let mut irqs = [0; 8];
            file.read(&mut irqs).expect("pskbd: failed to read irq:1");

            let data: u8;
            unsafe {
                asm!("in al, dx" : "={al}"(data) : "{dx}"(0x60) : : "intel", "volatile");
            }

            println!("pskbd: IRQ {}: {:X}", unsafe { *(irqs.as_ptr() as *const usize) }, data);

            file.write(&irqs).expect("pskbd: failed to write irq:1");
        }
    } else {
        unsafe {
            iopl(3).expect("psmsd: failed to get I/O permission");
            asm!("cli" :::: "intel", "volatile");
        }

        let mut file = File::open("irq:12").expect("psmsd: failed to open irq:12");

        println!("psmsd: Reading mouse IRQs");

        loop {
            let mut count = [0; 8];
            file.read(&mut count).expect("psmsd: failed to read irq:12");

            let data: u8;
            unsafe {
                asm!("in al, dx" : "={al}"(data) : "{dx}"(0x60) : : "intel", "volatile");
            }

            println!("psmsd: IRQ: {:X}", data);

            file.write(&count).expect("psmsd: failed to write irq:12");
        }
    }
}
