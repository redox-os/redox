use collections::VecDeque;
use core::str;
use spin::{Mutex, Once};

use context;
use syscall::error::*;
use syscall::scheme::Scheme;

/// Input
static INPUT: Once<Mutex<VecDeque<u8>>> = Once::new();

/// Initialize contexts, called if needed
fn init_input() -> Mutex<VecDeque<u8>> {
    Mutex::new(VecDeque::new())
}

/// Get the global schemes list, const
#[no_mangle]
pub extern fn debug_input(b: u8) {
    INPUT.call_once(init_input).lock().push_back(b)
}

pub struct DebugScheme;

impl Scheme for DebugScheme {
    fn open(&self, _path: &[u8], _flags: usize) -> Result<usize> {
        Ok(0)
    }

    fn dup(&self, _file: usize) -> Result<usize> {
        Ok(0)
    }

    /// Read the file `number` into the `buffer`
    ///
    /// Returns the number of bytes read
    fn read(&self, _file: usize, buf: &mut [u8]) -> Result<usize> {
        loop {
            let mut i = 0;
            {
                let mut input = INPUT.call_once(init_input).lock();
                while i < buf.len() && ! input.is_empty() {
                    buf[i] = input.pop_front().expect("debug_input lost byte");
                    i += 1;
                }
            }

            if i > 0 {
                return Ok(i);
            } else {
                unsafe { context::switch(); }
            }
        }
    }

    /// Write the `buffer` to the `file`
    ///
    /// Returns the number of bytes written
    fn write(&self, _file: usize, buffer: &[u8]) -> Result<usize> {
        //TODO: Write bytes, do not convert to str
        print!("{}", unsafe { str::from_utf8_unchecked(buffer) });
        Ok(buffer.len())
    }

    fn fsync(&self, _file: usize) -> Result<usize> {
        Ok(0)
    }

    /// Close the file `number`
    fn close(&self, _file: usize) -> Result<usize> {
        Ok(0)
    }
}
