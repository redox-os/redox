use std::fs::File;
use std::io::{Read, Write};
use std::mem;
use std::thread;

use keymap;

pub fn keyboard() {    
    let mut file = File::open("irq:1").expect("ps2d: failed to open irq:1");

    loop {
        let mut irqs = [0; 8];
        if file.read(&mut irqs).expect("ps2d: failed to read irq:1") >= mem::size_of::<usize>() {
            let data: u8;
            unsafe {
                asm!("in al, dx" : "={al}"(data) : "{dx}"(0x60) : : "intel", "volatile");
            }

            let (scancode, pressed) = if data >= 0x80 {
                (data - 0x80, false)
            } else {
                (data, true)
            };

            if pressed {
                print!("{}", keymap::get_char(scancode));
            }

            file.write(&irqs).expect("ps2d: failed to write irq:1");
        } else {
            thread::yield_now();
        }
    }
}
