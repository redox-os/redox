use redox::fs::file::Seek;
use redox::string::*;

pub struct Resource {
    path: String
}

impl Resource {
    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        Some(0)
    }

    pub fn write(&mut self, buf: &[u8]) -> Option<usize> {
        Some(0)
    }

    pub fn seek(&mut self, seek: Seek) -> Option<usize> {
        Some(0)
    }

    pub fn sync(&mut self) -> bool {
        true
    }
}

pub struct Scheme;

impl Scheme {
    pub fn new() -> Self {
        Scheme
    }

    pub fn open(&mut self, path: &str) -> Resource {
        Resource {
            path: path.to_string()
        }
    }
}
