use syscall::error::*;
use syscall::scheme::Scheme;

pub struct ZeroScheme;

impl Scheme for ZeroScheme {
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
        let mut i = 0;
        while i < buf.len() {
            buf[i] = 0;
            i += 1;
        }
        Ok(i)
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
