use std::cell::UnsafeCell;
use std::fs::File;
use std::io::{Read, Write, Result};
use std::mem;
use std::path::PathBuf;
use std::slice;

/// Redox domain socket
pub struct Socket {
    file: UnsafeCell<File>
}

impl Socket {
    pub fn open(path: &str) -> Result<Socket> {
        let file = try!(File::open(path));
        Ok(Socket {
            file: UnsafeCell::new(file)
        })
    }

    pub fn create(path: &str) -> Result<Socket> {
        let file = try!(File::create(path));
        Ok(Socket {
            file: UnsafeCell::new(file)
        })
    }

    pub fn dup(&self) -> Result<Socket> {
        let file = try!(unsafe { (*self.file.get()).dup() });
        Ok(Socket {
            file: UnsafeCell::new(file)
        })
    }

    pub fn path(&self) -> Result<PathBuf> {
        unsafe { (*self.file.get()).path() }
    }

    pub fn receive(&self, buf: &mut [u8]) -> Result<usize> {
        unsafe { (*self.file.get()).read(buf) }
    }

    pub fn receive_type<T: Copy>(&self, buf: &mut [T]) -> Result<usize> {
        self.receive(unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, buf.len() * mem::size_of::<T>()) }).map(|count| count/mem::size_of::<T>())
    }

    pub fn send(&self, buf: &[u8]) -> Result<usize> {
        unsafe { (*self.file.get()).write(buf) }
    }

    pub fn send_type<T: Copy>(&self, buf: &[T]) -> Result<usize> {
        self.send(unsafe { slice::from_raw_parts(buf.as_ptr() as *const u8, buf.len() * mem::size_of::<T>()) }).map(|count| count/mem::size_of::<T>())
    }
}
