use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;

use core::cmp::{min, max};

/// ARP scheme
pub mod arp;
/// Context scheme
pub mod context;
/// Debug scheme
pub mod debug;
/// Display Scheme
pub mod display;
/// Ethernet scheme
pub mod ethernet;
/// File scheme
pub mod file;
/// IP scheme
pub mod ip;
/// Memory scheme
pub mod memory;
/// Pseudo random generation scheme
pub mod random;
/// Time scheme
pub mod time;
/// Events scheme
pub mod events;

#[allow(unused_variables)]
pub trait KScheme {
    fn on_irq(&mut self, irq: u8) {

    }

    fn on_poll(&mut self) {

    }

    fn scheme(&self) -> &str {
        ""
    }

    fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
        None
    }
}

/// Resource seek
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
    fn dup(&self) -> Option<Box<Resource>> {
        None
    }
    /// Return the url of this resource
    fn url(&self) -> URL;
    // TODO: Make use of Write and Read trait
    /// Read data to buffer
    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        None
    }
    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        None
    }
    /// Seek
    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        None
    }
    /// Sync the resource
    fn sync(&mut self) -> bool {
        false
    }

    fn truncate(&mut self, len: usize) -> bool {
        false
    }

    //Helper functions
    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        let mut read = 0;
        loop {
            let mut bytes = [0; 1024];
            match self.read(&mut bytes) {
                Some(0) => return Some(read),
                None => return None,
                Some(count) => {
                    vec.push_all(&bytes[0..count]);
                    read += count;
                }
            }
        }
    }
}

/// An URL, see wiki
pub struct URL {
    pub string: String,
}

impl URL {
    /// Create a new empty URL
    pub fn new() -> Self {
        URL { string: String::new() }
    }

    /// Create an URL from a string literal
    pub fn from_str(url_str: &'static str) -> Self {
        return URL::from_string(&url_str.to_string());
    }

    /// Create an URL from `String`
    pub fn from_string(url_string: &String) -> Self {
        URL { string: url_string.clone() }
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
    pub fn open(&self) -> Option<Box<Resource>> {
        unsafe {
            return (*::session_ptr).open(&self);
        }
    }

    /// Return the scheme of this url
    pub fn scheme(&self) -> &str {
        &self.string[..self.string.find(':').unwrap_or(self.string.len())]
    }

    /// Get the reference (after the ':') of the url
    pub fn reference(&self) -> &str {
        &self.string[
        match self.string.find(':') {
            Some(pos) => pos + 1,
            None => self.string.len(),
        }
        ..]
    }

}

impl Clone for URL {
    fn clone(&self) -> Self {
        URL { string: self.string.clone() }
    }
}

/// A vector resource
pub struct VecResource {
    url: URL,
    vec: Vec<u8>,
    seek: usize,
}

impl VecResource {
    pub fn new(url: URL, vec: Vec<u8>) -> Self {
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
    fn dup(&self) -> Option<Box<Resource>> {
        Some(box VecResource {
            url: self.url.clone(),
            vec: self.vec.clone(),
            seek: self.seek,
        })
    }

    fn url(&self) -> URL {
        return self.url.clone();
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.vec.len() {
            match self.vec.get(self.seek) {
                Some(b) => buf[i] = *b,
                None => (),
            }
            self.seek += 1;
            i += 1;
        }
        return Some(i);
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
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
        return Some(i);
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        match pos {
            ResourceSeek::Start(offset) => self.seek = min(self.vec.len(), offset),
            ResourceSeek::Current(offset) =>
                self.seek = max(0, min(self.seek as isize, self.seek as isize + offset)) as usize,
            ResourceSeek::End(offset) =>
                self.seek =
                    max(0, min(self.seek as isize, self.vec.len() as isize + offset)) as usize,
        }
        return Some(self.seek);
    }

    fn sync(&mut self) -> bool {
        return true;
    }

    fn truncate(&mut self, len: usize) -> bool {
        while len > self.vec.len() {
            self.vec.push(0);
        }
        self.vec.truncate(len);
        self.seek = min(self.seek, self.vec.len());
        true
    }
}
