use common::slice::GetSlice;

use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;

use core::cmp::{min, max};

use syscall::{Error, O_CREAT, O_RDWR, O_TRUNC, EBADF, ENOENT};
use env;

/// Context scheme
pub mod context;
/// Debug scheme
pub mod debug;
/// Display Scheme
pub mod display;
/// File scheme
pub mod file;
/// Interrupt scheme
pub mod interrupt;
/// Memory scheme
pub mod memory;
/// Pipes
pub mod pipe;
/// Tests
pub mod test;

pub type Result<T> = ::core::result::Result<T, Error>;

#[allow(unused_variables)]
pub trait KScheme {
    fn on_irq(&mut self, irq: u8) {

    }

    fn on_poll(&mut self) {

    }

    fn scheme(&self) -> &str {
        ""
    }

    fn open<'a>(&'a mut self, url: &Url, flags: usize) -> Result<Box<Resource + 'a>> {
        Err(Error::new(ENOENT))
    }

    fn unlink(&mut self, url: &Url) -> Result<()> {
        Err(Error::new(ENOENT))
    }
}

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
    fn dup<'a>(&'a self) -> Result<Box<Resource + 'a>> {
        Err(Error::new(EBADF))
    }
    /// Return the url of this resource
    fn url(&self) -> Url;
    // TODO: Make use of Write and Read trait
    /// Read data to buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        Err(Error::new(EBADF))
    }
    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Err(Error::new(EBADF))
    }
    /// Seek
    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        Err(Error::new(EBADF))
    }
    /// Sync the resource
    fn sync(&mut self) -> Result<()> {
        Err(Error::new(EBADF))
    }

    fn truncate(&mut self, len: usize) -> Result<()> {
        Err(Error::new(EBADF))
    }

    // Helper functions
    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Result<usize> {
        let mut read = 0;
        loop {
            let mut bytes = [0; 1024];
            match self.read(&mut bytes) {
                Ok(0) => return Ok(read),
                Err(err) => return Err(err),
                Ok(count) => {
                    vec.push_all(bytes.get_slice(..count));
                    read += count;
                }
            }
        }
    }
}

/// An URL, see wiki
pub struct Url {
    pub string: String,
}

impl Url {
    /// Create a new empty URL
    pub fn new() -> Self {
        Url { string: String::new() }
    }

    /// Create an URL from a string literal
    pub fn from_str(url_str: &str) -> Self {
        Url::from_string(url_str.to_string())
    }

    /// Create an URL from `String`
    pub fn from_string(url_string: String) -> Self {
        Url { string: url_string }
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        self.string.clone()
    }

    /// Get the length of this URL
    pub fn len(&self) -> usize {
        self.string.len()
    }

    /// Open this URL (returns a resource)
    pub fn open(&self) -> Result<Box<Resource>> {
        env().open(&self, O_RDWR)
    }

    /// Create this URL (returns a resource)
    pub fn create(&self) -> Result<Box<Resource>> {
        env().open(&self, O_CREAT | O_RDWR | O_TRUNC)
    }

    /// Return the scheme of this url
    pub fn scheme(&self) -> &str {
        self.string.get_slice(..self.string.find(':'))
    }

    /// Get the reference (after the ':') of the url
    pub fn reference(&self) -> &str {
        self.string.get_slice(self.string.find(':').map(|a| a + 1)..)
    }

}

impl Clone for Url {
    fn clone(&self) -> Self {
        Url { string: self.string.clone() }
    }
}

/// A vector resource
pub struct VecResource {
    url: Url,
    vec: Vec<u8>,
    seek: usize,
}

impl VecResource {
    pub fn new(url: Url, vec: Vec<u8>) -> Self {
        VecResource {
            url: url,
            vec: vec,
            seek: 0,
        }
    }

    pub fn inner(&self) -> &Vec<u8> {
        return &self.vec;
    }
}

impl Resource for VecResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box VecResource {
            url: self.url.clone(),
            vec: self.vec.clone(),
            seek: self.seek,
        })
    }

    fn url(&self) -> Url {
        return self.url.clone();
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.vec.len() {
            match self.vec.get(self.seek) {
                Some(b) => buf[i] = *b,
                None => (),
            }
            self.seek += 1;
            i += 1;
        }
        return Ok(i);
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.vec.len() {
            self.vec[self.seek] = buf[i];
            self.seek += 1;
            i += 1;
        }
        while i < buf.len() {
            self.vec.push(buf[i]);
            self.seek += 1;
            i += 1;
        }
        return Ok(i);
    }

    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        match pos {
            ResourceSeek::Start(offset) => self.seek = min(self.vec.len(), offset),
            ResourceSeek::Current(offset) =>
                self.seek = max(0, min(self.seek as isize, self.seek as isize + offset)) as usize,
            ResourceSeek::End(offset) =>
                self.seek = max(0,
                                min(self.seek as isize,
                                    self.vec.len() as isize +
                                    offset)) as usize,
        }
        return Ok(self.seek);
    }

    fn sync(&mut self) -> Result<()> {
        Ok(())
    }

    fn truncate(&mut self, len: usize) -> Result<()> {
        while len > self.vec.len() {
            self.vec.push(0);
        }
        self.vec.truncate(len);
        self.seek = min(self.seek, self.vec.len());
        Ok(())
    }
}
