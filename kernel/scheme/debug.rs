use core::str;
use syscall::Result;
use super::Scheme;

pub struct DebugScheme;

impl Scheme for DebugScheme {
    fn open(&mut self, path: &[u8], flags: usize) -> Result<usize> {
        println!("DebugScheme::open: {}", unsafe { str::from_utf8_unchecked(path) });
        Ok(0)
    }

    /// Read the file `number` into the `buffer`
    ///
    /// Returns the number of bytes read
    fn read(&mut self, file: usize, buffer: &mut [u8]) -> Result<usize> {
        Ok(0)
    }

    /// Write the `buffer` to the `file`
    ///
    /// Returns the number of bytes written
    fn write(&mut self, file: usize, buffer: &[u8]) -> Result<usize> {
        //TODO: Write bytes, do not convert to str
        print!("{}", unsafe { str::from_utf8_unchecked(buffer) });
        Ok(buffer.len())
    }

    /// Close the file `number`
    fn close(&mut self, file: usize) -> Result<()> {
        Ok(())
    }
}
