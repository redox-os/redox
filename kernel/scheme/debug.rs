use core::str;

use syscall::Result;
use super::Scheme;

pub struct DebugScheme;

impl Scheme for DebugScheme {
    fn open(&mut self, _path: &[u8], _flags: usize) -> Result<usize> {
        Ok(0)
    }

    fn dup(&mut self, _file: usize) -> Result<usize> {
        Ok(0)
    }

    /// Read the file `number` into the `buffer`
    ///
    /// Returns the number of bytes read
    fn read(&mut self, _file: usize, _buffer: &mut [u8]) -> Result<usize> {
        Ok(0)
    }

    /// Write the `buffer` to the `file`
    ///
    /// Returns the number of bytes written
    fn write(&mut self, _file: usize, buffer: &[u8]) -> Result<usize> {
        //TODO: Write bytes, do not convert to str
        print!("{}", unsafe { str::from_utf8_unchecked(buffer) });
        Ok(buffer.len())
    }

    /// Close the file `number`
    fn close(&mut self, _file: usize) -> Result<()> {
        Ok(())
    }
}
