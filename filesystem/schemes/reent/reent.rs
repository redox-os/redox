use redox::Box;
use redox::fs::File;
use redox::string::{String, ToString};
use redox::io::{Read, Write, Seek, SeekFrom};

pub struct Resource {
    file: File
}

impl Resource {
    pub fn dup(&self) -> Option<Box<Self>> {
        match self.file.dup() {
            Some(file) => Some(box Resource {
                file: file
            }),
            None => None
        }
    }

    pub fn path(&self) -> Option<String> {
        self.file.path()
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        self.file.read(buf)
    }

    pub fn write(&mut self, buf: &[u8]) -> Option<usize> {
        self.file.write(buf)
    }

    pub fn seek(&mut self, seek: SeekFrom) -> Option<usize> {
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

    pub fn open(&mut self, path: &str, _: usize) -> Option<Box<Resource>> {
        match File::open(&("example:".to_string() + path)) {
            Some(file) => Some(box Resource {
                file: file
            }),
            None => None
        }
    }
}
