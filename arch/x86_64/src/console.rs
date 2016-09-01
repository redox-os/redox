use core::fmt::{self, Write};
use spin::Mutex;

use device::display::DISPLAY;
use device::serial::COM1;

pub static CONSOLE: Mutex<Console> = Mutex::new(Console::new());

pub struct Console;

impl Console {
    const fn new() -> Self {
        Console
    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        if let Some(ref mut display) = *DISPLAY.lock() {
            display.write(s.as_bytes());
            Ok(())
        } else {
            COM1.lock().write_str(s)
        }
    }
}
