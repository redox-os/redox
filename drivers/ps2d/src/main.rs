#![feature(asm)]

#[macro_use]
extern crate bitflags;
extern crate io;
extern crate syscall;

use std::thread;

use syscall::iopl;

mod controller;
mod keyboard;
mod keymap;
mod mouse;

fn main() {
    unsafe {
        iopl(3).expect("ps2d: failed to get I/O permission");
        asm!("cli" :::: "intel", "volatile");
    }

    let extra_packet = controller::Ps2::new().init();

    thread::spawn(|| {
        unsafe {
            iopl(3).expect("ps2d: failed to get I/O permission");
            asm!("cli" :::: "intel", "volatile");
        }

        keyboard::keyboard();
    });

    thread::spawn(move || {
        unsafe {
            iopl(3).expect("ps2d: failed to get I/O permission");
            asm!("cli" :::: "intel", "volatile");
        }

        mouse::mouse(extra_packet);
    });
}
