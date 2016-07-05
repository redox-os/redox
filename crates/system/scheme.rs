use core::ops::{Deref, DerefMut};
use core::{mem, slice};

use super::error::*;
use super::syscall::*;
use super::c_string_to_str;

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
            SYS_MKDIR => self.mkdir(c_string_to_str(packet.b as *const u8), packet.c),
            SYS_RMDIR => self.rmdir(c_string_to_str(packet.b as *const u8)),
            SYS_STAT => self.stat(c_string_to_str(packet.b as *const u8), unsafe { &mut *(packet.c as *mut Stat) }),
            SYS_UNLINK => self.unlink(c_string_to_str(packet.b as *const u8)),

            SYS_DUP => self.dup(packet.b),
            SYS_READ => self.read(packet.b, unsafe { slice::from_raw_parts_mut(packet.c as *mut u8, packet.d) }),
            SYS_WRITE => self.write(packet.b, unsafe { slice::from_raw_parts(packet.c as *const u8, packet.d) }),
            SYS_LSEEK => self.seek(packet.b, packet.c, packet.d),
            SYS_FPATH => self.fpath(packet.b, unsafe { slice::from_raw_parts_mut(packet.c as *mut u8, packet.d) }),
            SYS_FSTAT => self.fstat(packet.b, unsafe { &mut *(packet.c as *mut Stat) }),
            SYS_FSYNC => self.fsync(packet.b),
            SYS_FTRUNCATE => self.ftruncate(packet.b, packet.c),
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
    fn mkdir(&mut self, path: &str, mode: usize) -> Result<usize> {
        Err(Error::new(ENOENT))
    }

    #[allow(unused_variables)]
    fn rmdir(&mut self, path: &str) -> Result<usize> {
        Err(Error::new(ENOENT))
    }

    #[allow(unused_variables)]
    fn stat(&mut self, path: &str, stat: &mut Stat) -> Result<usize> {
        Err(Error::new(ENOENT))
    }

    #[allow(unused_variables)]
    fn unlink(&mut self, path: &str) -> Result<usize> {
        Err(Error::new(ENOENT))
    }

    /* Resource operations */
    #[allow(unused_variables)]
    fn dup(&mut self, old_id: usize) -> Result<usize> {
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
    fn fpath(&self, id: usize, buf: &mut [u8]) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn fstat(&self, id: usize, stat: &mut Stat) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn fsync(&mut self, id: usize) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn ftruncate(&mut self, id: usize, len: usize) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn close(&mut self, id: usize) -> Result<usize> {
        Err(Error::new(EBADF))
    }
}
