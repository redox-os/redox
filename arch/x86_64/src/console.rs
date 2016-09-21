use core::fmt::{self, Write};
use spin::Mutex;

use device::serial::COM1;

pub static CONSOLE: Mutex<Console> = Mutex::new(Console);

pub struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        COM1.lock().write_str(s)
    }
}
