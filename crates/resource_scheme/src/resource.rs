use syscall::data::Stat;
use syscall::error::*;

pub trait Resource {
    /// Duplicate the resource
    /// Returns `EPERM` if the operation is not supported.
    fn dup(&self) -> Result<Box<Self>> {
        Err(Error::new(EPERM))
    }

    /// Return the path of this resource
    /// Returns `EPERM` if the operation is not supported.
    fn path(&self, _buf: &mut [u8]) -> Result<usize> {
        Err(Error::new(EPERM))
    }

    /// Read data to buffer
    /// Returns `EPERM` if the operation is not supported.
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize> {
        Err(Error::new(EPERM))
    }

    /// Write to resource
    /// Returns `EPERM` if the operation is not supported.
    fn write(&mut self, _buf: &[u8]) -> Result<usize> {
        Err(Error::new(EPERM))
    }

    /// Seek to the given offset
    /// Returns `ESPIPE` if the operation is not supported.
    fn seek(&mut self, _pos: usize, _whence: usize) -> Result<usize> {
        Err(Error::new(ESPIPE))
    }

    /// Get informations about the resource, such as mode and size
    /// Returns `EPERM` if the operation is not supported.
    fn stat(&self, _stat: &mut Stat) -> Result<usize> {
        Err(Error::new(EPERM))
    }

    /// Sync all buffers
    /// Returns `EPERM` if the operation is not supported.
    fn sync(&mut self) -> Result<usize> {
        Err(Error::new(EPERM))
    }

    /// Truncate to the given length
    /// Returns `EPERM` if the operation is not supported.
    fn truncate(&mut self, _len: usize) -> Result<usize> {
        Err(Error::new(EPERM))
    }
}
