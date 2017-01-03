use syscall::error::*;
use syscall::scheme::Scheme;

pub struct NullScheme;

impl Scheme for NullScheme {
    fn open(&self, _path: &[u8], _flags: usize, _uid: u32, _gid: u32) -> Result<usize> {
        Ok(0)
    }

    fn dup(&self, _file: usize, _buf: &[u8]) -> Result<usize> {
        Ok(0)
    }

    /// Read the file `number` into the `buffer`
    ///
    /// Returns the number of bytes read
    fn read(&self, _file: usize, _buf: &mut [u8]) -> Result<usize> {
        Ok(0)
    }

    /// Write the `buffer` to the `file`
    ///
    /// Returns the number of bytes written
    fn write(&self, _file: usize, buffer: &[u8]) -> Result<usize> {
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
