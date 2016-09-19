#![feature(asm)]

extern crate syscall;

use std::fs::File;
use std::io::{Read, Write};
use std::mem;
use std::thread;

use syscall::iopl;

mod keymap;

fn keyboard() {
    let mut file = File::open("irq:1").expect("pskbd: failed to open irq:1");

    loop {
        let mut irqs = [0; 8];
        if file.read(&mut irqs).expect("pskbd: failed to read irq:1") >= mem::size_of::<usize>() {
            let data: u8;
            unsafe {
                asm!("in al, dx" : "={al}"(data) : "{dx}"(0x60) : : "intel", "volatile");
            }

            let (scancode, pressed) = if data >= 0x80 {
                (data - 0x80, false)
            } else {
                (data, true)
            };
            println!("pskbd: IRQ {}: {:X}: {:X}: {}: {}", unsafe { *(irqs.as_ptr() as *const usize) }, data, scancode, keymap::get_char(scancode), pressed);

            file.write(&irqs).expect("pskbd: failed to write irq:1");
        } else {
            thread::yield_now();
        }
    }
}

fn mouse() {
    let mut file = File::open("irq:12").expect("psmsd: failed to open irq:12");

    loop {
        let mut irqs = [0; 8];
        if file.read(&mut irqs).expect("psmsd: failed to read irq:12") >= mem::size_of::<usize>() {
            let data: u8;
            unsafe {
                asm!("in al, dx" : "={al}"(data) : "{dx}"(0x60) : : "intel", "volatile");
            }

            println!("psmsd: IRQ {}: {:X}", unsafe { *(irqs.as_ptr() as *const usize) }, data);

            file.write(&irqs).expect("psmsd: failed to write irq:12");
        } else {
            thread::yield_now();
        }
    }
}

fn main() {
    thread::spawn(|| {
        unsafe {
            iopl(3).expect("pskbd: failed to get I/O permission");
            asm!("cli" :::: "intel", "volatile");
        }

        keyboard();
    });

    thread::spawn(|| {
        unsafe {
            iopl(3).expect("psmsd: failed to get I/O permission");
            asm!("cli" :::: "intel", "volatile");
        }

        mouse();
    });
}
