use core::fmt::{self, Write};
use spin::Mutex;

use device::display;
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
        if let Some(ref mut console) = *display::CONSOLE.lock() {
            if let Some(ref mut display) = *display::DISPLAY.lock() {
                console.write(s.as_bytes(), |event| {
                    display.event(event);
                });
            }

            Ok(())
        } else {
            COM1.lock().write_str(s)
        }
    }
}
