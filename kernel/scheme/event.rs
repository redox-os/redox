use core::{mem, str};

use arch::interrupt::irq::{ACKS, COUNTS, acknowledge};
use syscall::error::*;
use syscall::scheme::Scheme;

pub struct EventScheme;

impl Scheme for EventScheme {
    fn open(&self, _path: &[u8], _flags: usize) -> Result<usize> {
        Ok(
    }

    fn dup(&self, file: usize) -> Result<usize> {
        Ok(file)
    }

    fn read(&self, file: usize, buffer: &mut [u8]) -> Result<usize> {
        Ok(0)
    }

    fn write(&self, file: usize, buffer: &[u8]) -> Result<usize> {
        Ok(0)
    }

    fn fsync(&self, _file: usize) -> Result<usize> {
        Ok(0)
    }

    fn close(&self, _file: usize) -> Result<usize> {
        Ok(0)
    }
}
