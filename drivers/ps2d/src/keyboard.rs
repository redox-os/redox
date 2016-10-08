use std::fs::File;
use std::io::{Read, Write};
use std::mem;
use std::thread;

use keymap;

pub fn keyboard() {
    let mut file = File::open("irq:1").expect("ps2d: failed to open irq:1");
    let mut input = File::open("display:input").expect("ps2d: failed to open display:input");

    let mut ctrl = false;
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

            if scancode == 0x1D {
                ctrl = pressed;
            } else if scancode == 0x2A {
                lshift = pressed;
            } else if scancode == 0x36 {
                rshift = pressed;
            } else if pressed {
                match scancode {
                    f @ 0x3B ... 0x44 => { // F1 through F10
                        input.write(&[(f - 0x3B) + 0xF4]).unwrap();
                    },
                    0x57 => { // F11
                        input.write(&[0xFE]).unwrap();
                    },
                    0x58 => { // F12
                        input.write(&[0xFF]).unwrap();
                    },
                    0x47 => { // Home
                        input.write(b"\x1B[H").unwrap();
                    },
                    0x48 => { // Up
                        input.write(b"\x1B[A").unwrap();
                    },
                    0x49 => { // Page up
                        input.write(b"\x1B[5~").unwrap();
                    },
                    0x4B => { // Left
                        input.write(b"\x1B[D").unwrap();
                    },
                    0x4D => { // Right
                        input.write(b"\x1B[C").unwrap();
                    },
                    0x4F => { // End
                        input.write(b"\x1B[F").unwrap();
                    },
                    0x50 => { // Down
                        input.write(b"\x1B[B").unwrap();
                    },
                    0x51 => { // Page down
                        input.write(b"\x1B[6~").unwrap();
                    },
                    0x52 => { // Insert
                        input.write(b"\x1B[2~").unwrap();
                    },
                    0x53 => { // Delete
                        input.write(b"\x1B[3~").unwrap();
                    },
                    _ => {
                        let c = if ctrl {
                            match keymap::get_char(scancode, false) {
                                c @ 'a' ... 'z' => ((c as u8 - b'a') + b'\x01') as char,
                                c => c
                            }
                        } else {
                            keymap::get_char(scancode, lshift || rshift)
                        };

                        if c != '\0' {
                            input.write(&[c as u8]).unwrap();
                        }
                    }
                }
            }
        } else {
            thread::yield_now();
        }
    }
}
