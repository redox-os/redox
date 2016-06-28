use core::fmt;
use drivers::io::{Io, Pio};

pub struct SerialConsole {
    status: Pio<u8>,
    data: Pio<u8>
}

impl SerialConsole {
    pub fn new() -> SerialConsole {
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
        //self.write(s.as_bytes());

        {
            let console = unsafe { &mut *::env().console.get() };
            console.write(s.as_bytes());
        }

        Ok(())
    }
}

/// Debug to console
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        use $crate::core::fmt::Write;
        let _ = write!($crate::common::debug::SerialConsole::new(), $($arg)*);
    });
}

/// Debug new line to console
#[macro_export]
macro_rules! debugln {
    ($fmt:expr) => (debug!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (debug!(concat!($fmt, "\n"), $($arg)*));
}
