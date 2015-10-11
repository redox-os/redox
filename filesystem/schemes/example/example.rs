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
        write!(io::stdout(), "Read {} bytes from {}\n", buf.len(), self.path);
        Some(0)
    }

    pub fn write(&mut self, buf: &[u8]) -> Option<usize> {
        write!(io::stdout(), "Write {} bytes to {}\n", buf.len(), self.path);
        Some(0)
    }

    pub fn seek(&mut self, seek: Seek) -> Option<usize> {
        match seek {
            Seek::Start(offset) => {
                write!(io::stdout(), "Seek to Start({}) in {}\n", offset, self.path);
            },
            Seek::Current(offset) => {
                write!(io::stdout(), "Seek to Current({}) in {}\n", offset, self.path);
            },
            Seek::End(offset) => {
                write!(io::stdout(), "Seek to End({}) in {}\n", offset, self.path);
            }
        }
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
        write!(io::stdout(), "New example scheme\n");
        box Scheme
    }

    pub fn open(&mut self, path: &str) -> Box<Resource> {
        write!(io::stdout(), "Open {}\n", path);
        box Resource {
            path: path.to_string()
        }
    }
}
