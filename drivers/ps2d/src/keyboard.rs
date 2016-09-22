use std::fs::File;
use std::io::{Read, Write};
use std::mem;
use std::thread;

use keymap;

pub fn keyboard() {
    let mut file = File::open("irq:1").expect("ps2d: failed to open irq:1");
    let mut input = File::open("display:input").expect("ps2d: failed to open display:input");

    let mut lshift = false;
    let mut rshift = false;
    loop {
        let mut irqs = [0; 8];
        if file.read(&mut irqs).expect("ps2d: failed to read irq:1") >= mem::size_of::<usize>() {
            let data: u8;
            unsafe {
                asm!("in al, dx" : "={al}"(data) : "{dx}"(0x60) : : "intel", "volatile");
            }

            file.write(&irqs).expect("ps2d: failed to write irq:1");

            let (scancode, pressed) = if data >= 0x80 {
                (data - 0x80, false)
            } else {
                (data, true)
            };

            if scancode == 0x2A {
                lshift = pressed;
            } else if scancode == 0x36 {
                rshift = pressed;
            } else if pressed {
                let c = keymap::get_char(scancode, lshift || rshift);
                if c != '\0' {
                    print!("{}", c);
                    input.write(&[c as u8]).expect("ps2d: failed to write input");
                }
            }
        } else {
            thread::yield_now();
        }
    }
}
