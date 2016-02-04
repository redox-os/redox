
use core::ops::{Deref, DerefMut};
use core::{mem, ptr, slice, str};

use super::error::*;
use super::syscall::*;

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
            slice::from_raw_parts(self as *const Packet as *const u8, mem::size_of::<Packet>()) as &[u8]
        }
    }
}

impl DerefMut for Packet {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self as *mut Packet as *mut u8, mem::size_of::<Packet>()) as &mut [u8]
        }
    }
}

pub trait Scheme {
    fn handle(&mut self, packet: &mut Packet) {
        packet.a = Error::mux(match packet.a {
            SYS_OPEN => self.open(c_string_to_str(packet.b as *const u8), packet.c, packet.d),
            SYS_UNLINK => self.unlink(c_string_to_str(packet.b as *const u8)),
            SYS_MKDIR => self.mkdir(c_string_to_str(packet.b as *const u8), packet.c),

            SYS_FPATH => self.path(packet.b, unsafe { slice::from_raw_parts_mut(packet.c as *mut u8, packet.d) }),
            SYS_READ => self.read(packet.b, unsafe { slice::from_raw_parts_mut(packet.c as *mut u8, packet.d) }),
            SYS_WRITE => self.write(packet.b, unsafe { slice::from_raw_parts(packet.c as *const u8, packet.d) }),
            SYS_LSEEK => self.seek(packet.b, packet.c, packet.d),
            SYS_FSYNC => self.sync(packet.b),
            SYS_FTRUNCATE => self.truncate(packet.b, packet.c),
            SYS_CLOSE => self.close(packet.b),

            _ => Err(Error::new(ENOSYS))
        });
    }

    /* Scheme operations */

    #[allow(unused_variables)]
    fn open(&mut self, path: &str, flags: usize, mode: usize) -> Result<usize> {
        Err(Error::new(ENOENT))
    }

    #[allow(unused_variables)]
    fn unlink(&mut self, path: &str) -> Result<usize> {
        Err(Error::new(ENOENT))
    }

    #[allow(unused_variables)]
    fn mkdir(&mut self, path: &str, mode: usize) -> Result<usize> {
        Err(Error::new(ENOENT))
    }

    /* Resource operations */
    #[allow(unused_variables)]
    fn path(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn read(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn write(&mut self, id: usize, buf: &[u8]) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn seek(&mut self, id: usize, pos: usize, whence: usize) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn sync(&mut self, id: usize) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn truncate(&mut self, id: usize, len: usize) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn close(&mut self, id: usize) -> Result<usize> {
        Err(Error::new(EBADF))
    }
}
