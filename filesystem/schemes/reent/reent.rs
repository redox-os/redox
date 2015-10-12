use redox::Box;
use redox::fs::file::{File, Seek};
use redox::string::*;
use redox::io::{Read, Write};

pub struct Resource {
    file: File
}

impl Resource {
    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        self.file.read(buf)
    }

    pub fn write(&mut self, buf: &[u8]) -> Option<usize> {
        self.file.write(buf)
    }

    pub fn seek(&mut self, seek: Seek) -> Option<usize> {
        self.file.seek(seek)
    }

    pub fn sync(&mut self) -> bool {
        self.file.sync()
    }
}

pub struct Scheme;

impl Scheme {
    pub fn new() -> Box<Self> {
        box Scheme
    }

    pub fn open(&mut self, path: &str) -> Box<Resource> {
        box Resource {
            file: File::open(&("example:".to_string() + path))
        }
    }
}
