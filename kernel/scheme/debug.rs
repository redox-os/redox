use core::str;
use core::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use spin::Once;

use context;
use sync::WaitQueue;
use syscall::error::*;
use syscall::flag::EVENT_READ;
use syscall::scheme::Scheme;

pub static DEBUG_SCHEME_ID: AtomicUsize = ATOMIC_USIZE_INIT;

/// Input queue
static INPUT: Once<WaitQueue<u8>> = Once::new();

/// Initialize input queue, called if needed
fn init_input() -> WaitQueue<u8> {
    WaitQueue::new()
}

/// Add to the input queue
#[no_mangle]
pub extern fn debug_input(b: u8) {
    let len = INPUT.call_once(init_input).send(b);

    context::event::trigger(DEBUG_SCHEME_ID.load(Ordering::SeqCst), 0, EVENT_READ, len);
}

pub struct DebugScheme;

impl Scheme for DebugScheme {
    fn open(&self, _path: &[u8], _flags: usize, _uid: u32, _gid: u32) -> Result<usize> {
        Ok(0)
    }

    fn dup(&self, _file: usize, _buf: &[u8]) -> Result<usize> {
        Ok(0)
    }

    /// Read the file `number` into the `buffer`
    ///
    /// Returns the number of bytes read
    fn read(&self, _file: usize, buf: &mut [u8]) -> Result<usize> {
        Ok(INPUT.call_once(init_input).receive_into(buf, true))
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
