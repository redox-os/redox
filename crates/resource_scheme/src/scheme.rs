use std::slice;

use syscall::data::{Packet, Stat};
use syscall::error::*;
use syscall::number::*;

use super::Resource;

pub trait ResourceScheme<T: Resource> {
    fn open_resource(&self, path: &[u8], flags: usize, uid: u32, gid: u32) -> Result<Box<T>>;

    fn handle(&self, packet: &mut Packet) {
        packet.a = Error::mux(match packet.a {
            SYS_OPEN => self.open(unsafe { slice::from_raw_parts(packet.b as *const u8, packet.c) }, packet.d, packet.uid, packet.gid),
            SYS_MKDIR => self.mkdir(unsafe { slice::from_raw_parts(packet.b as *const u8, packet.c) }, packet.d as u16, packet.uid, packet.gid),
            SYS_RMDIR => self.rmdir(unsafe { slice::from_raw_parts(packet.b as *const u8, packet.c) }, packet.uid, packet.gid),
            SYS_UNLINK => self.unlink(unsafe { slice::from_raw_parts(packet.b as *const u8, packet.c) }, packet.uid, packet.gid),

            SYS_DUP => self.dup(packet.b),
            SYS_READ => self.read(packet.b, unsafe { slice::from_raw_parts_mut(packet.c as *mut u8, packet.d) }),
            SYS_WRITE => self.write(packet.b, unsafe { slice::from_raw_parts(packet.c as *const u8, packet.d) }),
            SYS_LSEEK => self.seek(packet.b, packet.c, packet.d),
            SYS_FEVENT => self.fevent(packet.b, packet.c),
            SYS_FPATH => self.fpath(packet.b, unsafe { slice::from_raw_parts_mut(packet.c as *mut u8, packet.d) }),
            SYS_FSTAT => self.fstat(packet.b, unsafe { &mut *(packet.c as *mut Stat) }),
            SYS_FSYNC => self.fsync(packet.b),
            SYS_FTRUNCATE => self.ftruncate(packet.b, packet.c),
            SYS_CLOSE => self.close(packet.b),

            _ => Err(Error::new(ENOSYS))
        });
    }

    /* Scheme operations */
    fn open(&self, path: &[u8], flags: usize, uid: u32, gid: u32) -> Result<usize> {
        let resource = self.open_resource(path, flags, uid, gid)?;
        let resource_ptr = Box::into_raw(resource);
        Ok(resource_ptr as usize)
    }

    #[allow(unused_variables)]
    fn mkdir(&self, path: &[u8], mode: u16, uid: u32, gid: u32) -> Result<usize> {
        Err(Error::new(ENOENT))
    }

    #[allow(unused_variables)]
    fn rmdir(&self, path: &[u8], uid: u32, gid: u32) -> Result<usize> {
        Err(Error::new(ENOENT))
    }

    #[allow(unused_variables)]
    fn unlink(&self, path: &[u8], uid: u32, gid: u32) -> Result<usize> {
        Err(Error::new(ENOENT))
    }

    /* Resource operations */
    fn dup(&self, old_id: usize) -> Result<usize> {
        let old = unsafe { &*(old_id as *const T) };
        let resource = old.dup()?;
        let resource_ptr = Box::into_raw(resource);
        Ok(resource_ptr as usize)
    }

    fn read(&self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let mut resource = unsafe { &mut *(id as *mut T) };
        resource.read(buf)
    }

    fn write(&self, id: usize, buf: &[u8]) -> Result<usize> {
        let mut resource = unsafe { &mut *(id as *mut T) };
        resource.write(buf)
    }

    fn seek(&self, id: usize, pos: usize, whence: usize) -> Result<usize> {
        let mut resource = unsafe { &mut *(id as *mut T) };
        resource.seek(pos, whence)
    }

    #[allow(unused_variables)]
    fn fevent(&self, id: usize, flags: usize) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    fn fpath(&self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let resource = unsafe { &*(id as *const T) };
        resource.path(buf)
    }

    fn fstat(&self, id: usize, stat: &mut Stat) -> Result<usize> {
        let resource = unsafe { &*(id as *const T) };
        resource.stat(stat)
    }

    fn fsync(&self, id: usize) -> Result<usize> {
        let mut resource = unsafe { &mut *(id as *mut T) };
        resource.sync()
    }

    fn ftruncate(&self, id: usize, len: usize) -> Result<usize> {
        let mut resource = unsafe { &mut *(id as *mut T) };
        resource.truncate(len)
    }

    fn close(&self, id: usize) -> Result<usize> {
        let resource = unsafe { Box::from_raw(id as *mut T) };
        drop(resource);
        Ok(0)
    }
}
