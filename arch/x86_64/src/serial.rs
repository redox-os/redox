use core::fmt;
use spin::Mutex;

use super::io::{Io, Pio};

static SERIAL_PORT: Mutex<SerialPort> = Mutex::new(SerialPort::new());

struct SerialPort {
    status: Pio<u8>,
    data: Pio<u8>
}

impl SerialPort {
    pub const fn new() -> SerialPort {
        SerialPort {
            status: Pio::new(0x3F8 + 5),
            data: Pio::new(0x3F8)
        }
    }

    pub fn write(&mut self, bytes: &[u8]) {
        for byte in bytes.iter() {
            while !self.status.readf(0x20) {}
            self.data.write(*byte);

            if *byte == 8 {
                while !self.status.readf(0x20) {}
                self.data.write(0x20);

                while !self.status.readf(0x20) {}
                self.data.write(8);
            }
        }
    }
}

pub struct SerialConsole;

impl fmt::Write for SerialConsole {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        SERIAL_PORT.lock().write(s.as_bytes());

        Ok(())
    }
}
