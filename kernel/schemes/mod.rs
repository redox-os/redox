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

    fn open<'a, 'b: 'a>(&'a mut self, url: Url<'b>, flags: usize) -> Result<Box<Resource + 'a>> {
        Err(Error::new(ENOENT))
    }

    fn unlink<'a>(&mut self, url: Url<'a>) -> Result<()> {
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
#[derive(Clone, Copy)]
pub struct Url<'a> {
    pub scheme: &'a str,
    pub reference: &'a str,
}

impl<'a> Url<'a> {
    /// Create a new empty URL
    pub const fn new() -> Self {
        Url {
            scheme: "",
            reference: "",
        }
    }

    /// Create an URL from a string literal
    pub fn from_str(url_str: &'a str) -> Self {
        url_str.into()
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        self.scheme.to_string() + ":" + self.reference
    }

    /// Get the length of this URL
    pub fn len(&self) -> usize {
        self.scheme.len() + self.reference.len() + 1
    }

    /// Open this URL (returns a resource)
    pub fn open<'b>(self) -> Result<Box<Resource + 'b>> {
        env().open(self, O_RDWR)
    }

    /// Create this URL (returns a resource)
    pub fn create(self) -> Result<Box<Resource>> {
        env().open(self, O_CREAT | O_RDWR | O_TRUNC)
    }
}

impl<'a> From<&'a str> for Url<'a> {
    fn from(s: &'a str) -> Url<'a> {
        let splitter = s.find(':');

        Url {
            scheme: s.get_slice(..splitter),
            reference: s.get_slice(splitter.map(|a| a + 1)..),
        }
    }
}

/// A vector resource
pub struct VecResource<'a> {
    url: Url<'a>,
    vec: Vec<u8>, // TODO it seems that the length of this vec is often known. Consider using a slice instead.
    seek: usize,
}

impl<'a> VecResource<'a> {
    pub fn new(url: Url<'a>, vec: Vec<u8>) -> Self {
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

impl<'a> Resource for VecResource<'a> {
    fn dup<'b>(&'b self) -> Result<Box<Resource + 'b>> {
        Ok(box VecResource {
            url: self.url,
            vec: self.vec.clone(),
            seek: self.seek,
        })
    }

    fn url(&self) -> Url {
        self.url
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
