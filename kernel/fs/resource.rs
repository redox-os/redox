use alloc::boxed::Box;

use system::error::{Error, Result, EPERM, ESPIPE};
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
    /// Returns `EPERM` if the operation is not supported.
    fn dup(&self) -> Result<Box<Resource>> {
        Err(Error::new(EPERM))
    }

    /// Return the path of this resource
    /// Returns `EPERM` if the operation is not supported.
    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        Err(Error::new(EPERM))
    }

    /// Read data to buffer
    /// Returns `EPERM` if the operation is not supported.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        Err(Error::new(EPERM))
    }

    /// Write to resource
    /// Returns `EPERM` if the operation is not supported.
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Err(Error::new(EPERM))
    }

    /// Seek to the given offset
    /// Returns `ESPIPE` if the operation is not supported.
    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        Err(Error::new(ESPIPE))
    }

    /// Get informations about the resource, such as mode and size
    /// Returns `EPERM` if the operation is not supported.
    fn stat(&self, stat: &mut Stat) -> Result<()> {
        Err(Error::new(EPERM))
    }

    /// Sync all buffers
    /// Returns `EPERM` if the operation is not supported.
    fn sync(&mut self) -> Result<()> {
        Err(Error::new(EPERM))
    }

    /// Truncate to the given length
    /// Returns `EPERM` if the operation is not supported.
    fn truncate(&mut self, len: usize) -> Result<()> {
        Err(Error::new(EPERM))
    }
}
