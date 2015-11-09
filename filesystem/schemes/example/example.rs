use redox::Box;
use redox::string::{String, ToString};
use redox::io::{self, SeekFrom, Write};

pub struct Resource {
    path: String
}

impl Resource {
    pub fn dup(&self) -> Option<Box<Self>> {
        Some(box Resource {
            path: self.path.clone()
        })
    }

    pub fn path(&self) -> Option<String> {
        Some(self.path.clone())
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        write!(io::stdout(), "Read {} bytes from {}\n", buf.len(), self.path);
        Some(0)
    }

    pub fn write(&mut self, buf: &[u8]) -> Option<usize> {
        write!(io::stdout(), "Write {} bytes to {}\n", buf.len(), self.path);
        Some(0)
    }

    pub fn seek(&mut self, seek: SeekFrom) -> Option<usize> {
        match seek {
            SeekFrom::Start(offset) => {
                write!(io::stdout(), "Seek to Start({}) in {}\n", offset, self.path);
            },
            SeekFrom::Current(offset) => {
                write!(io::stdout(), "Seek to Current({}) in {}\n", offset, self.path);
            },
            SeekFrom::End(offset) => {
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

    pub fn poll(&mut self) -> bool {
        false
    }

    pub fn open(&mut self, path: &str, _: usize) -> Option<Box<Resource>> {
        write!(io::stdout(), "Open {}\n", path);
        Some(box Resource {
            path: path.to_string()
        })
    }
}
