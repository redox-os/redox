#![crate_type="lib"]

use std::ops::{Deref, DerefMut};
use std::{ptr, slice, str};

pub use std::io::{Result, SeekFrom};
pub use std::syscall::*;


/// Helper function for handling C strings, please do not copy it or make it pub or change it
fn c_string_to_str<'a>(ptr: *const u8) -> &'a str {
    if ptr > 0 as *const u8 {
        let mut len = 0;
        while unsafe { ptr::read(ptr.offset(len as isize)) } > 0 {
            len += 1;
        }

        unsafe { str::from_utf8_unchecked(slice::from_raw_parts(ptr, len)) }
    } else {
        ""
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(packed)]
pub struct Packet {
    pub id: usize,
    pub a: usize,
    pub b: usize,
    pub c: usize,
    pub d: usize
}

impl Deref for Packet {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self as *const Packet as *const u8, std::mem::size_of::<Packet>()) as &[u8]
        }
    }
}

impl DerefMut for Packet {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self as *mut Packet as *mut u8, std::mem::size_of::<Packet>()) as &mut [u8]
        }
    }
}

pub trait Resource {
    #[allow(unused_variables)]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        Err(SysError::new(EBADF))
    }

    #[allow(unused_variables)]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Err(SysError::new(EBADF))
    }

    #[allow(unused_variables)]
    fn seek(&mut self, pos: SeekFrom) -> Result<usize> {
        Err(SysError::new(EBADF))
    }
}

pub trait Scheme {
    fn handle(&mut self, packet: &mut Packet) {
        packet.a = SysError::mux(match packet.a {
            SYS_OPEN => self.open(c_string_to_str(packet.b as *const u8), packet.c, packet.d),
            SYS_UNLINK => self.unlink(c_string_to_str(packet.b as *const u8)),
            _ => Err(SysError::new(ENOSYS))
        });
    }

    #[allow(unused_variables)]
    fn open(&mut self, path: &str, flags: usize, mode: usize) -> Result<usize> {
        Err(SysError::new(ENOENT))
    }

    #[allow(unused_variables)]
    fn unlink(&mut self, path: &str) -> Result<usize> {
        Err(SysError::new(ENOENT))
    }

    #[allow(unused_variables)]
    fn mkdir(&mut self, path: &str, mode: usize) -> Result<usize> {
        Err(SysError::new(ENOENT))
    }
}
