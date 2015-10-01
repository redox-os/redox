use common::string::*;
use common::vec::*;

use syscall::call::*;

/// A Unix-style file
pub struct File {
    path: String,
    fd: usize
}

impl File {
    /// Open a new file using a path
    // TODO: Why &String and not String
    pub fn open(path: &String) -> File {
        unsafe {
            let c_str: *const u8 = path.to_c_str();
            let ret = File {
                path: path.clone(),
                fd: sys_open(c_str, 0, 0)
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

    /// Read a file to a buffer
    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        unsafe {
            let count = sys_read(self.fd, buf.as_mut_ptr(), buf.len());
            if count == 0xFFFFFFFF {
                Option::None
            }else{
                Option::Some(count)
            }
        }
    }

    /// Read the file to the end
    pub fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
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

    /*
    pub fn seek(&mut self, pos: Seek) -> Option<usize> {
        Option::None
    }
    */

    /// Flush the io
    pub fn flush(&mut self) -> bool {
        // TODO
        false
    }
}

impl Drop for File {
    fn drop(&mut self){
        unsafe {
            sys_close(self.fd);
        }
    }
}
