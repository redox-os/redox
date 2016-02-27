use alloc::boxed::Box;

use system::error::{Error, Result, EBADF};
use system::syscall::Stat;

/// Resource seek
#[derive(Copy, Clone, Debug)]
pub enum ResourceSeek {
    /// Start point
    Start(usize),
    /// Current point
    Current(isize),
    /// End point
    End(isize),
}

/// A system resource
#[allow(unused_variables)]
pub trait Resource {
    /// Duplicate the resource
    fn dup(&self) -> Result<Box<Resource>> {
        Err(Error::new(EBADF))
    }

    /// Return the path of this resource
    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    /// Read data to buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    /// Seek to the given offset
    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    fn stat(&self, stat: &mut Stat) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    /// Sync all buffers
    fn sync(&mut self) -> Result<()> {
        Err(Error::new(EBADF))
    }

    /// Truncate to the given length
    fn truncate(&mut self, len: usize) -> Result<()> {
        Err(Error::new(EBADF))
    }
}
