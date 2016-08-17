use core::fmt;
use spin::Mutex;

use super::io::{Io, Pio};

pub static SERIAL_CONSOLE: Mutex<SerialConsole> = Mutex::new(SerialConsole::new());

pub struct SerialConsole {
    status: Pio<u8>,
    data: Pio<u8>
}

impl SerialConsole {
    pub const fn new() -> SerialConsole {
        SerialConsole {
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

impl fmt::Write for SerialConsole {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.write(s.as_bytes());

        Ok(())
    }
}
