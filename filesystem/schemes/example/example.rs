use core::fmt::Write;

use redox::Box;
use redox::fs::file::Seek;
use redox::string::*;
use redox::io;

pub struct Resource {
    path: String
}

impl Resource {
    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        write!(io::stdout(), "Read {} bytes to {}\n", buf.len(), self.path);
        Some(0)
    }

    pub fn write(&mut self, buf: &[u8]) -> Option<usize> {
        write!(io::stdout(), "Write {} bytes from {}\n", buf.len(), self.path);
        Some(0)
    }

    pub fn seek(&mut self, seek: Seek) -> Option<usize> {
        write!(io::stdout(), "Seek to TODO in {}\n", self.path);
        Some(0)
    }

    pub fn sync(&mut self) -> bool {
        write!(io::stdout(), "Sync {}\n", self.path);
        true
    }
}

pub struct Scheme;

impl Scheme {
    pub fn new() -> Box<Self> {
        box Scheme
    }

    pub fn open(&mut self, path: &str) -> Box<Resource> {
        write!(io::stdout(), "Open {}\n", path);
        box Resource {
            path: path.to_string()
        }
    }
}
