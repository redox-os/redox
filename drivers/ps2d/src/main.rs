#![feature(asm)]

#[macro_use]
extern crate bitflags;
extern crate event;
extern crate io;
extern crate orbclient;
extern crate syscall;

use std::fs::File;
use std::io::{Read, Write, Result};
use std::os::unix::io::AsRawFd;
use std::{mem, thread};

use event::EventQueue;
use orbclient::{KeyEvent, MouseEvent};
use syscall::iopl;

mod controller;
mod keymap;

bitflags! {
    flags MousePacketFlags: u8 {
        const LEFT_BUTTON = 1,
        const RIGHT_BUTTON = 1 << 1,
        const MIDDLE_BUTTON = 1 << 2,
        const ALWAYS_ON = 1 << 3,
        const X_SIGN = 1 << 4,
        const Y_SIGN = 1 << 5,
        const X_OVERFLOW = 1 << 6,
        const Y_OVERFLOW = 1 << 7
    }
}

struct Ps2d {
    input: File,
    lshift: bool,
    rshift: bool,
    packets: [u8; 4],
    packet_i: usize,
    extra_packet: bool
}

impl Ps2d {
    fn new(input: File, extra_packet: bool) -> Self {
        Ps2d {
            input: input,
            lshift: false,
            rshift: false,
            packets: [0; 4],
            packet_i: 0,
            extra_packet: extra_packet
        }
    }

    fn handle(&mut self, keyboard: bool, data: u8) {
        if keyboard {
            let (scancode, pressed) = if data >= 0x80 {
                (data - 0x80, false)
            } else {
                (data, true)
            };

            if scancode == 0x2A {
                self.lshift = pressed;
            } else if scancode == 0x36 {
                self.rshift = pressed;
            }

            self.input.write(&KeyEvent {
                character: keymap::get_char(scancode, self.lshift || self.rshift),
                scancode: scancode,
                pressed: pressed
            }.to_event()).expect("ps2d: failed to write key event");
        } else {
            self.packets[self.packet_i] = data;
            self.packet_i += 1;

            let flags = MousePacketFlags::from_bits_truncate(self.packets[0]);
            if ! flags.contains(ALWAYS_ON) {
                println!("MOUSE MISALIGN {:X}", self.packets[0]);

                self.packets = [0; 4];
                self.packet_i = 0;
            } else if self.packet_i >= self.packets.len() || (!self.extra_packet && self.packet_i >= 3) {
                if ! flags.contains(X_OVERFLOW) && ! flags.contains(Y_OVERFLOW) {
                    let mut dx = self.packets[1] as i32;
                    if flags.contains(X_SIGN) {
                        dx -= 0x100;
                    }

                    let mut dy = -(self.packets[2] as i32);
                    if flags.contains(Y_SIGN) {
                        dy += 0x100;
                    }

                    let _extra = if self.extra_packet {
                        self.packets[3]
                    } else {
                        0
                    };

                    self.input.write(&MouseEvent {
                        x: dx,
                        y: dy,
                        left_button: flags.contains(LEFT_BUTTON),
                        middle_button: flags.contains(MIDDLE_BUTTON),
                        right_button: flags.contains(RIGHT_BUTTON)
                    }.to_event()).expect("ps2d: failed to write mouse event");
                } else {
                    println!("ps2d: overflow {:X} {:X} {:X} {:X}", self.packets[0], self.packets[1], self.packets[2], self.packets[3]);
                }

                self.packets = [0; 4];
                self.packet_i = 0;
            }
        }
    }
}

fn main() {
    thread::spawn(|| {
        unsafe {
            iopl(3).expect("ps2d: failed to get I/O permission");
            asm!("cli" : : : : "intel", "volatile");
        }

        let input = File::open("display:input").expect("ps2d: failed to open display:input");

        let extra_packet = controller::Ps2::new().init();

        let mut ps2d = Ps2d::new(input, extra_packet);

        let mut event_queue = EventQueue::<(bool, u8)>::new().expect("ps2d: failed to create event queue");

        let mut key_irq = File::open("irq:1").expect("ps2d: failed to open irq:1");

        event_queue.add(key_irq.as_raw_fd(), move |_count: usize| -> Result<Option<(bool, u8)>> {
            let mut irq = [0; 8];
            if key_irq.read(&mut irq)? >= mem::size_of::<usize>() {
                let data: u8;
                unsafe {
                    asm!("in al, dx" : "={al}"(data) : "{dx}"(0x60) : : "intel", "volatile");
                }

                key_irq.write(&irq)?;

                Ok(Some((true, data)))
            } else {
                Ok(None)
            }
        }).expect("ps2d: failed to poll irq:1");

        let mut mouse_irq = File::open("irq:12").expect("ps2d: failed to open irq:12");

        event_queue.add(mouse_irq.as_raw_fd(), move |_count: usize| -> Result<Option<(bool, u8)>> {
            let mut irq = [0; 8];
            if mouse_irq.read(&mut irq)? >= mem::size_of::<usize>() {
                let data: u8;
                unsafe {
                    asm!("in al, dx" : "={al}"(data) : "{dx}"(0x60) : : "intel", "volatile");
                }

                mouse_irq.write(&irq)?;

                Ok(Some((false, data)))
            } else {
                Ok(None)
            }
        }).expect("ps2d: failed to poll irq:12");

        for (keyboard, data) in event_queue.trigger_all(0).expect("ps2d: failed to trigger events") {
            ps2d.handle(keyboard, data);
        }

        loop {
            let (keyboard, data) = event_queue.run().expect("ps2d: failed to handle events");
            ps2d.handle(keyboard, data);
        }
    });
}
