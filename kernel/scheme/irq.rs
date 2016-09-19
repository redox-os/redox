use core::{mem, str};

use arch::interrupt::irq::COUNTS;
use context;
use syscall::{Error, Result};
use super::Scheme;

pub struct IrqScheme;

impl Scheme for IrqScheme {
    fn open(&mut self, path: &[u8], _flags: usize) -> Result<usize> {
        let path_str = str::from_utf8(path).or(Err(Error::NoEntry))?;
        let id = path_str.parse::<usize>().or(Err(Error::NoEntry))?;
        if id < COUNTS.lock().len() {
            Ok(id)
        } else {
            Err(Error::NoEntry)
        }
    }

    fn dup(&mut self, file: usize) -> Result<usize> {
        Ok(file)
    }

    fn read(&mut self, file: usize, buffer: &mut [u8]) -> Result<usize> {
        // Ensures that the length of the buffer is larger than the size of a usize
        if buffer.len() >= mem::size_of::<usize>() {
            let current = COUNTS.lock()[file];
            loop {
                let next = COUNTS.lock()[file];
                if next != current {
                    // Safe if the length of the buffer is larger than the size of a usize
                    assert!(buffer.len() >= mem::size_of::<usize>());
                    unsafe { *(buffer.as_mut_ptr() as *mut usize) = next };
                    return Ok(mem::size_of::<usize>());
                } else {
                    // Safe if all locks have been dropped
                    unsafe { context::switch(); }
                }
            }
        } else {
            Err(Error::InvalidValue)
        }
    }

    fn write(&mut self, _file: usize, _buffer: &[u8]) -> Result<usize> {
        Err(Error::NotPermitted)
    }

    fn fsync(&mut self, file: usize) -> Result<()> {
        Ok(())
    }

    fn close(&mut self, file: usize) -> Result<()> {
        Ok(())
    }
}
