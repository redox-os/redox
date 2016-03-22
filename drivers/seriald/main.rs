extern crate io;
extern crate system;

use io::{Io, Pio};

use system::syscall::sys_iopl;

use std::thread;

#[repr(packed)]
struct SerialInfo {
    pub ports: [u16; 4],
}

/// Serial
pub struct Serial {
    pub data: Pio<u8>,
    pub status: Pio<u8>,
    pub irq: u8,
    pub escape: bool,
    pub cursor_control: bool,
}

impl Serial {
    /// Create new
    pub fn new(port: u16, irq: u8) -> Serial {
        Pio::<u8>::new(port + 1).write(0x00);
        Pio::<u8>::new(port + 3).write(0x80);
        Pio::<u8>::new(port + 0).write(0x03);
        Pio::<u8>::new(port + 1).write(0x00);
        Pio::<u8>::new(port + 3).write(0x03);
        Pio::<u8>::new(port + 2).write(0xC7);
        Pio::<u8>::new(port + 4).write(0x0B);
        Pio::<u8>::new(port + 1).write(0x01);

        Serial {
            data: Pio::<u8>::new(port),
            status: Pio::<u8>::new(port + 5),
            irq: irq,
            escape: false,
            cursor_control: false,
        }
    }

    pub fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes.iter() {
            while ! self.status.readf(0x20) {}
            self.data.write(byte);
        }
    }
}

fn main() {
    thread::spawn(|| {
        unsafe { sys_iopl(3).unwrap() };
        let mut serial = Serial::new(0x3F8, 4);
        serial.write(b"TEST\n");
        thread::sleep_ms(10000);
    });
}
