use string::*;

use core::ptr;

use io::{Read, Write, Seek, SeekFrom};

use syscall::{sys_alloc, sys_unalloc, sys_open, sys_close, sys_fpath, sys_read, sys_write, sys_lseek, sys_fsync};

/// A Unix-style file
pub struct File {
    /// The id for the file
    fd: usize,
}

impl File {
    /// Open a new file using a path
    // TODO: Return Option<File>
    pub fn open(path: &str) -> Self {
        unsafe {
            let c_str = sys_alloc(path.len() + 1) as *mut u8;
            if path.len() > 0 {
                ptr::copy(path.as_ptr(), c_str, path.len());
            }
            ptr::write(c_str.offset(path.len() as isize), 0);

            let ret = File {
                fd: sys_open(c_str, 0, 0),
            };

            sys_unalloc(c_str as usize);

            ret
        }
    }

    /// Get the canonical path of the file
    pub fn path(&self, buf: &mut [u8]) -> Option<usize> {
        unsafe {
            let count = sys_fpath(self.fd, buf.as_mut_ptr(), buf.len());
            if count == 0xFFFFFFFF {
                None
            } else {
                Some(count)
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
            if count == 0xFFFFFFFF {
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
            if count == 0xFFFFFFFF {
                Option::None
            } else {
                Option::Some(count)
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
        if position == 0xFFFFFFFF {
            Option::None
        } else {
            Option::Some(position)
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
