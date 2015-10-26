use core::usize;

use io::{Read, Write, Seek, SeekFrom};
use string::{String, ToString};
use vec::Vec;

use syscall::{sys_open, sys_dup, sys_close, sys_execve, sys_fpath, sys_read, sys_write, sys_lseek, sys_fsync};

/// A Unix-style file
pub struct File {
    /// The id for the file
    fd: usize,
}

impl File {
    pub fn exec(path: &str) -> bool {
        unsafe {
            sys_execve((path.to_string() + "\0").as_ptr()) == 0
        }
    }

    /// Open a new file using a path
    pub fn open(path: &str) -> Option<File> {
        unsafe {
            let fd = sys_open((path.to_string() + "\0").as_ptr(), 0, 0);
            if fd == usize::MAX {
                None
            }else{
                Some(File {
                    fd: fd
                })
            }
        }
    }

    /// Duplicate the file
    pub fn dup(&self) -> Option<File> {
        unsafe{
            let new_fd = sys_dup(self.fd);
            if new_fd == usize::MAX {
                None
            } else {
                Some(File {
                    fd: new_fd
                })
            }
        }
    }

    /// Get the canonical path of the file
    pub fn path(&self) -> Option<String> {
        unsafe {
            let mut buf: [u8; 4096] = [0; 4096];
            let count = sys_fpath(self.fd, buf.as_mut_ptr(), buf.len());
            if count == usize::MAX {
                None
            } else {
                Some(String::from_utf8_unchecked(Vec::from(&buf[0..count])))
            }
        }
    }

    /// Flush the io
    pub fn sync(&mut self) -> bool {
        unsafe { sys_fsync(self.fd) == 0 }
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        unsafe {
            let count = sys_read(self.fd, buf.as_mut_ptr(), buf.len());
            if count == usize::MAX {
                None
            } else {
                Some(count)
            }
        }
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        unsafe {
            let count = sys_write(self.fd, buf.as_ptr(), buf.len());
            if count == usize::MAX {
                None
            } else {
                Some(count)
            }
        }
    }
}

impl Seek for File {
    /// Seek a given position
    fn seek(&mut self, pos: SeekFrom) -> Option<usize> {
        let (whence, offset) = match pos {
            SeekFrom::Start(offset) => (0, offset as isize),
            SeekFrom::Current(offset) => (1, offset),
            SeekFrom::End(offset) => (2, offset),
        };

        let position = unsafe { sys_lseek(self.fd, offset, whence) };
        if position == usize::MAX {
            None
        } else {
            Some(position)
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
