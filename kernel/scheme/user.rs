use alloc::arc::{Arc, Weak};
use alloc::boxed::Box;

use collections::{BTreeMap, VecDeque};

use core::cell::Cell;
use core::mem::size_of;
use core::ops::DerefMut;
use core::{ptr, slice};

use syscall::{Call, Error, Result};

use super::Scheme;

/// UserScheme has to be wrapped
pub struct UserScheme {
    next_id: Cell<usize>,
    todo: VecDeque<(usize, (usize, usize, usize, usize))>,
    done: BTreeMap<usize, (usize, usize, usize, usize)>,
}

impl UserScheme {
    fn call(&self, a: Call, b: usize, c: usize, d: usize) -> Result<usize> {
        let id = self.next_id.get();

        //TODO: What should be done about collisions in self.todo or self.done?
        let mut next_id = id + 1;
        if next_id <= 0 {
            next_id = 1;
        }
        self.next_id.set(next_id);

        // println!("{} {}: {} {} {:X} {:X} {:X}", UserScheme.name, id, a, ::syscall::name(a), b, c, d);

        Ok(0)
    }

    fn capture(&self, mut physical_address: usize, size: usize, writeable: bool) -> Result<usize> {
        Ok(0)
    }

    fn release(&self, virtual_address: usize) {

    }
}

impl Scheme for UserScheme {
    fn open(&mut self, path: &[u8], flags: usize) -> Result<usize> {
        let virtual_address = try!(self.capture(path.as_ptr() as usize, path.len(), false));

        let result = self.call(Call::Open, virtual_address, path.len(), flags);

        self.release(virtual_address);

        result
    }

    /*
    fn mkdir(&mut self, path: &str, flags: usize) -> Result<()> {
        let virtual_address = try!(self.capture(path.as_ptr() as usize, path.len(), false));

        let result = self.call(Call::MkDir, virtual_address, path.len(), flags);

        self.release(virtual_address);

        result.and(Ok(()))
    }

    fn rmdir(&mut self, path: &str) -> Result<()> {
        let virtual_address = try!(self.capture(path.as_ptr() as usize, path.len(), false));

        let result = self.call(SYS_RMDIR, virtual_address, path.len(), 0);

        self.release(virtual_address);

        result.and(Ok(()))
    }

    fn unlink(&mut self, path: &str) -> Result<()> {
        let virtual_address = try!(self.capture(path.as_ptr() as usize, path.len(), false));

        let result = self.call(SYS_UNLINK, virtual_address, path.len(), 0);

        self.release(virtual_address);

        result.and(Ok(()))
    }
    */

    /// Duplicate the resource
    fn dup(&mut self, file: usize) -> Result<usize> {
        self.call(Call::Dup, file, 0, 0)
    }

    /*
    /// Return the URL of this resource
    fn path(&self, file: usize, buf: &mut [u8]) -> Result <usize> {
        let contexts = unsafe { & *::env().contexts.get() };
        let current = try!(contexts.current());
        if let Ok(physical_address) = current.translate(buf.as_mut_ptr() as usize, buf.len()) {
            let offset = physical_address % 4096;

            let virtual_address = try!(self.capture(physical_address - offset, buf.len() + offset, true));

            let result = self.call(SYS_FPATH, file, virtual_address + offset, buf.len());

            //println!("Read {:X} mapped from {:X} to {:X} offset {} length {} size {} result {:?}", physical_address, buf.as_ptr() as usize, virtual_address + offset, offset, buf.len(), virtual_size, result);

            self.release(virtual_address);

            result
        } else {
            println!("{}:{} fault {:X} {}", file!(), line!(), buf.as_ptr() as usize, buf.len());
            Err(Error::Fault)
        }
    }
    */

    /// Read data to buffer
    fn read(&mut self, file: usize, buf: &mut [u8]) -> Result<usize> {
        /*
        let contexts = unsafe { & *::env().contexts.get() };
        let current = try!(contexts.current());
        if let Ok(physical_address) = current.translate(buf.as_mut_ptr() as usize, buf.len()) {
            let offset = physical_address % 4096;

            let virtual_address = try!(self.capture(physical_address - offset, buf.len() + offset, true));

            let result = self.call(Call::Read, file, virtual_address + offset, buf.len());

            //println!("Read {:X} mapped from {:X} to {:X} offset {} length {} size {} result {:?}", physical_address, buf.as_ptr() as usize, virtual_address + offset, offset, buf.len(), virtual_size, result);

            self.release(virtual_address);

            result
        } else */
        {
            println!("{}:{} fault {:X} {}", file!(), line!(), buf.as_ptr() as usize, buf.len());
            Err(Error::Fault)
        }
    }

    /// Write to resource
    fn write(&mut self, file: usize, buf: &[u8]) -> Result<usize> {
        /*
        let contexts = unsafe { & *::env().contexts.get() };
        let current = try!(contexts.current());
        if let Ok(physical_address) = current.translate(buf.as_ptr() as usize, buf.len()) {
            let offset = physical_address % 4096;

            let virtual_address = try!(self.capture(physical_address - offset, buf.len() + offset, false));

            let result = self.call(Call::Write, file, virtual_address + offset, buf.len());

            // println!("Write {:X} mapped from {:X} to {:X} offset {} length {} result {:?}", physical_address, buf.as_ptr() as usize, virtual_address + offset, offset, buf.len(), result);

            self.release(virtual_address);

            result
        } else */
        {
            println!("{}:{} fault {:X} {}", file!(), line!(), buf.as_ptr() as usize, buf.len());
            Err(Error::Fault)
        }
    }

    /*
    /// Seek
    fn seek(&mut self, file: usize, pos: ResourceSeek) -> Result<usize> {
        let (whence, offset) = match pos {
            ResourceSeek::Start(offset) => (SEEK_SET, offset as usize),
            ResourceSeek::Current(offset) => (SEEK_CUR, offset as usize),
            ResourceSeek::End(offset) => (SEEK_END, offset as usize)
        };

        self.call(SYS_LSEEK, file, offset, whence)
    }

    /// Stat the resource
    fn stat(&self, file: usize, stat: &mut Stat) -> Result<()> {
        let buf = unsafe { slice::from_raw_parts_mut(stat as *mut Stat as *mut u8, size_of::<Stat>()) };

        let contexts = unsafe { & *::env().contexts.get() };
        let current = try!(contexts.current());
        if let Ok(physical_address) = current.translate(buf.as_mut_ptr() as usize, buf.len()) {
            let offset = physical_address % 4096;

            let virtual_address = try!(self.capture(physical_address - offset, buf.len() + offset, true));

            let result = self.call(SYS_FSTAT, file, virtual_address + offset, 0);

            self.release(virtual_address);

            result.and(Ok(()))
        } else {
            println!("{}:{} fault {:X} {}", file!(), line!(), buf.as_ptr() as usize, buf.len());
            Err(Error::Fault)
        }
    }
    */

    /// Sync the resource
    fn fsync(&mut self, file: usize) -> Result<()> {
        self.call(Call::FSync, file, 0, 0).and(Ok(()))
    }

    /*
    /// Truncate the resource
    fn truncate(&mut self, file: usize, len: usize) -> Result<()> {
        self.call(SYS_FTRUNCATE, file, len, 0).and(Ok(()))
    }
    */

    fn close(&mut self, file: usize) -> Result<()> {
        self.call(Call::Close, file, 0, 0).and(Ok(()))
    }
}
