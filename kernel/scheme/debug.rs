use collections::VecDeque;
use core::str;
use core::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use spin::{Mutex, Once};

use context;
use syscall::error::*;
use syscall::flag::EVENT_READ;
use syscall::scheme::Scheme;

pub static DEBUG_SCHEME_ID: AtomicUsize = ATOMIC_USIZE_INIT;

/// Input
static INPUT: Once<Mutex<VecDeque<u8>>> = Once::new();

/// Initialize contexts, called if needed
fn init_input() -> Mutex<VecDeque<u8>> {
    Mutex::new(VecDeque::new())
}

/// Get the global schemes list, const
#[no_mangle]
pub extern fn debug_input(b: u8) {
    let len = {
        let mut input = INPUT.call_once(init_input).lock();
        input.push_back(b);
        input.len()
    };

    context::event::trigger(DEBUG_SCHEME_ID.load(Ordering::SeqCst), 0, EVENT_READ, len);
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
                unsafe { context::switch(); } //TODO: Block
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

    fn fevent(&self, _file: usize, _flags: usize) -> Result<usize> {
        Ok(0)
    }

    fn fsync(&self, _file: usize) -> Result<usize> {
        Ok(0)
    }

    /// Close the file `number`
    fn close(&self, _file: usize) -> Result<usize> {
        Ok(0)
    }
}
