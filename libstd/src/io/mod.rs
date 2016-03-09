//! IO

use fmt;
use string::String;
use vec::Vec;
use thread;
pub use system::error::Error;
use system::syscall::{sys_read, sys_write};

pub mod prelude;

pub type Result<T> = ::core::result::Result<T, Error>;

pub struct Bytes<R: Read> {
    reader: R,
}

impl<R: Read> Iterator for Bytes<R> {
    type Item = Result<u8>;

    fn next(&mut self) -> Option<Result<u8>> {
        let mut byte = [0];
        match self.reader.read(&mut byte) {
            Ok(0) => None,
            Ok(_) => Some(Ok(byte[0])),
            Err(err) => Some(Err(err))
        }
    }
}

/// Types you can read
pub trait Read {
    /// Read a file to a buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Read the file to the end
    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Result<usize> {
        let mut read = 0;
        loop {
            let mut buf = [0; 4096];
            match self.read(&mut buf) {
                Ok(0) => return Ok(read),
                Err(err) => return Err(err),
                Ok(count) => {
                    vec.extend_from_slice(&buf[0..count]);
                    read += count;
                }
            }
        }
    }

    /// Read the file to a string
    fn read_to_string(&mut self, string: &mut String) -> Result<usize> {
        let mut read = 0;
        loop {
            let mut buf = [0; 4096];
            match self.read(&mut buf) {
                Ok(0) => return Ok(read),
                Err(err) => return Err(err),
                Ok(count) => {
                    unsafe { string.as_mut_vec().extend_from_slice(&buf[.. count]); }
                    read += count;
                }
            }
        }
    }

    /// Return an iterator of the bytes
    fn bytes(self) -> Bytes<Self> where Self: Sized {
        Bytes {
            reader: self,
        }
    }
}

/// Types you can write
pub trait Write {
    /// Write to the file
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    //Attempt to write entire buffer
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        let mut i = 0;
        while i < buf.len() {
            let count = try!(self.write(&buf[i..]));
            if count > 0 {
                i += count;
            } else {
                thread::yield_now();
            }
        }
        Ok(())
    }

    /// Write a format to the file
    fn write_fmt(&mut self, args: fmt::Arguments) -> Result<()> {
        match self.write(fmt::format(args).as_bytes()) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

/// Seek Location
pub enum SeekFrom {
    /// The start point
    Start(u64),
    /// The current point
    Current(i64),
    /// The end point
    End(i64),
}

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
}

pub fn copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> Result<u64> where R: Read, W: Write {
    let mut copied = 0;
    loop {
        let mut bytes = [0; 4096];
        match reader.read(&mut bytes) {
            Ok(0) => return Ok(copied),
            Err(err) => return Err(err),
            Ok(count) => match writer.write(&bytes[.. count]){
                Ok(0) => return Ok(copied),
                Err(err) => return Err(err),
                Ok(count) => copied += count as u64
            }
        }
    }
}

/// Standard Input
pub struct Stdin;

/// Create a standard input
pub fn stdin() -> Stdin {
    Stdin
}

impl Stdin {
    pub fn lock(&self) -> StdinLock {
        StdinLock
    }

    pub fn read_line(&mut self, string: &mut String) -> Result<usize> {
        let mut i = 0;
        loop {
            let mut byte = [0];
            match sys_read(0, &mut byte) {
                Ok(0) => return Ok(i),
                Ok(_) => {
                    unsafe { string.as_mut_vec().push(byte[0]) };
                    i += 1;
                    if byte[0] == b'\n' {
                        return Ok(i);
                    }
                },
                Err(err) => return Err(err)
            }
        }
    }
}

/// Read implementation for standard input
impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        sys_read(0, buf)
    }
}

/// Standard Input lock
//TODO: Implement locking
pub struct StdinLock;

/// Read implementation for standard input lock
impl Read for StdinLock {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        sys_read(0, buf)
    }
}

/// Standard Output
pub struct Stdout;

/// Create a standard output
pub fn stdout() -> Stdout {
    Stdout
}

impl Stdout {
    pub fn lock(&self) -> StdoutLock {
        StdoutLock
    }
}

/// Write implementation for standard output
impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        sys_write(1, buf)
    }
}

/// Standard Output lock
//TODO: Implement locking
pub struct StdoutLock;

/// Write implementation for standard output lock
impl Write for StdoutLock {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        sys_write(1, buf)
    }
}

/// Standard Error
pub struct Stderr;

/// Create a standard error
pub fn stderr() -> Stderr {
    Stderr
}

impl Stderr {
    pub fn lock(&self) -> StderrLock {
        StderrLock
    }
}

/// Write implementation for standard error
impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        sys_write(2, buf)
    }
}

/// Standard Error lock
//TODO: Implement locking
pub struct StderrLock;

/// Write implementation for standard error lock
impl Write for StderrLock {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        sys_write(2, buf)
    }
}

#[allow(unused_must_use)]
pub fn _print(args: fmt::Arguments) {
    stdout().write_fmt(args);
}
