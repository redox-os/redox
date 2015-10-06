use collections::string::*;
use collections::vec::{IntoIter, Vec};

use core::ptr;

use syscall::call::*;

/// File seek
pub enum Seek {
    /// The start point
    Start(usize),
    /// The current point
    Current(isize),
    /// The end point
    End(isize),
}

/// A Unix-style file
pub struct File {
    /// The path to the file
    path: String,
    /// The id for the file
    fd: usize,
}

impl File {
    /// Open a new file using a path
    // TODO: Why &String and not String
    // TODO: Return Option<File>
    pub fn open(path: &str) -> Self {
        unsafe {
            let c_str = sys_alloc(path.len() + 1) as *mut u8;
            if path.len() > 0 {
                ptr::copy(path.as_ptr(), c_str, path.len());
            }
            ptr::write(c_str.offset(path.len() as isize), 0);

            let ret = File {
                path: path.to_string(),
                fd: sys_open(c_str, 0, 0),
            };

            sys_unalloc(c_str as usize);

            ret
        }
    }

    /// Return the url to the file
    pub fn url(&self) -> String {
        //TODO
        self.path.clone()
    }


    /// Write to the file
    pub fn write(&mut self, buf: &[u8]) -> Option<usize> {
        unsafe {
            let count = sys_write(self.fd, buf.as_ptr(), buf.len());
            if count == 0xFFFFFFFF {
                Option::None
            } else {
                Option::Some(count)
            }
        }
    }

    /// Seek a given position
    pub fn seek(&mut self, pos: Seek) -> Option<usize> {
        let (whence, offset) = match pos {
            Seek::Start(offset) => (0, offset as isize),
            Seek::Current(offset) => (1, offset),
            Seek::End(offset) => (2, offset),
        };

        let position = unsafe { sys_lseek(self.fd, offset, whence) };
        if position == 0xFFFFFFFF {
            Option::None
        } else {
            Option::Some(position)
        }
    }

    /// Flush the io
    pub fn sync(&mut self) -> bool {
        unsafe { sys_fsync(self.fd) == 0 }
    }
}

pub trait Read {

    /// Read a file to a buffer
    fn read(&mut self, buf: &mut [u8]) -> Option<usize>;

    /// Read the file to the end
    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        let mut read = 0;
        loop {
            let mut bytes = [0; 1024];
            match self.read(&mut bytes) {
                Option::Some(0) => return Option::Some(read),
                Option::None => return Option::None,
                Option::Some(count) => {
                    for i in 0..count {
                        vec.push(bytes[i]);
                    }
                    read += count;
                }
            }
        }
    }
    /// Return an iterator of the bytes
    fn bytes(&mut self) -> IntoIter<u8> {
        // TODO: This is only a temporary implementation. Make this read one byte at a time.
        let mut buf = Vec::new();
        self.read_to_end(&mut buf);

        buf.into_iter()
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        unsafe {
            let count = sys_read(self.fd, buf.as_mut_ptr(), buf.len());
            if count == 0xFFFFFFFF {
                Option::None
            } else {
                Option::Some(count)
            }
        }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            sys_close(self.fd);
        }
    }
}
