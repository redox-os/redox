use alloc::arc::{Arc, Weak};
use alloc::boxed::Box;

use collections::String;
use collections::borrow::ToOwned;

use core::cell::Cell;
use core::mem::size_of;
use core::ops::DerefMut;
use core::{ptr, slice};

use arch::context::{Context, ContextMemory};

use sync::{WaitMap, WaitQueue};

use system::error::{Error, Result, EBADF, EFAULT, EINVAL, ENODEV, ESPIPE};
use system::scheme::Packet;
use system::syscall::{SYS_CLOSE, SYS_DUP, SYS_FPATH, SYS_FSTAT, SYS_FSYNC, SYS_FTRUNCATE,
                    SYS_OPEN, SYS_LSEEK, SEEK_SET, SEEK_CUR, SEEK_END, SYS_MKDIR,
                    SYS_READ, SYS_WRITE, SYS_RMDIR, SYS_STAT, SYS_UNLINK, Stat};

use super::{Resource, ResourceSeek, KScheme, Url};

struct SchemeInner {
    name: String,
    context: *mut Context,
    next_id: Cell<usize>,
    todo: WaitQueue<Packet>,
    done: WaitMap<usize, (usize, usize, usize, usize)>,
}

impl SchemeInner {
    fn new(name: &str, context: *mut Context) -> SchemeInner {
        SchemeInner {
            name: name.to_owned(),
            context: context,
            next_id: Cell::new(1),
            todo: WaitQueue::new(),
            done: WaitMap::new(),
        }
    }

    fn call(inner: &Weak<SchemeInner>, a: usize, b: usize, c: usize, d: usize) -> Result<usize> {
        if let Some(scheme) = inner.upgrade() {
            let id = scheme.next_id.get();

            //TODO: What should be done about collisions in self.todo or self.done?
            let mut next_id = id + 1;
            if next_id <= 0 {
                next_id = 1;
            }
            scheme.next_id.set(next_id);

            scheme.todo.send(Packet {
                id: id,
                a: a,
                b: b,
                c: c,
                d: d
            });
            Error::demux(scheme.done.receive(&id).0)
        } else {
            Err(Error::new(ENODEV))
        }
    }

    fn capture(inner: &Weak<SchemeInner>, mut physical_address: usize, size: usize, writeable: bool) -> Result<usize> {
        if let Some(scheme) = inner.upgrade() {
            if physical_address >= 0x80000000 {
                physical_address -= 0x80000000;
            }
            unsafe {
                let mmap = &mut *(*scheme.context).mmap.get();
                let virtual_address = mmap.next_mem();
                mmap.memory.push(ContextMemory {
                    physical_address: physical_address,
                    virtual_address: virtual_address,
                    virtual_size: size,
                    writeable: writeable,
                    allocated: false,
                });
                return Ok(virtual_address);
            }
        } else {
            return Err(Error::new(ENODEV));
        }
    }

    fn release(inner: &Weak<SchemeInner>, virtual_address: usize) {
        if let Some(scheme) = inner.upgrade() {
            unsafe {
                let mmap = &mut *(*scheme.context).mmap.get();
                if let Ok(mut mem) = mmap.get_mem_mut(virtual_address) {
                    mem.virtual_size = 0;
                }
                mmap.clean_mem();
            }
        }
    }
}

impl Drop for SchemeInner {
    fn drop(&mut self) {
        unsafe { &mut *::env().schemes.get() }.retain(|scheme| scheme.scheme() != self.name);
    }
}

pub struct SchemeResource {
    inner: Weak<SchemeInner>,
    file_id: usize,
}

impl SchemeResource {
    fn call(&self, a: usize, b: usize, c: usize, d: usize) -> Result<usize> {
        SchemeInner::call(&self.inner, a, b, c, d)
    }

    fn capture(&self, physical_address: usize, size: usize, writeable: bool) -> Result<usize> {
        SchemeInner::capture(&self.inner, physical_address, size, writeable)
    }

    fn release(&self, virtual_address: usize){
        SchemeInner::release(&self.inner, virtual_address);
    }
}

impl Resource for SchemeResource {
    /// Duplicate the resource
    fn dup(&self) -> Result<Box<Resource>> {
        let file_id = try!(self.call(SYS_DUP, self.file_id, 0, 0));
        Ok(Box::new(SchemeResource {
            inner: self.inner.clone(),
            file_id: file_id
        }))
    }

    /// Return the url of this resource
    fn path(&self, buf: &mut [u8]) -> Result <usize> {
        let contexts = unsafe { & *::env().contexts.get() };
        let current = try!(contexts.current());
        if let Ok(physical_address) = current.translate(buf.as_mut_ptr() as usize, buf.len()) {
            let offset = physical_address % 4096;

            let virtual_address = try!(self.capture(physical_address - offset, buf.len() + offset, true));

            let result = self.call(SYS_FPATH, self.file_id, virtual_address + offset, buf.len());

            //debugln!("Read {:X} mapped from {:X} to {:X} offset {} length {} size {} result {:?}", physical_address, buf.as_ptr() as usize, virtual_address + offset, offset, buf.len(), virtual_size, result);

            self.release(virtual_address);

            result
        } else {
            debugln!("{}:{} fault {:X} {}", file!(), line!(), buf.as_ptr() as usize, buf.len());
            Err(Error::new(EFAULT))
        }
    }

    /// Read data to buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let contexts = unsafe { & *::env().contexts.get() };
        let current = try!(contexts.current());
        if let Ok(physical_address) = current.translate(buf.as_mut_ptr() as usize, buf.len()) {
            let offset = physical_address % 4096;

            let virtual_address = try!(self.capture(physical_address - offset, buf.len() + offset, true));

            let result = self.call(SYS_READ, self.file_id, virtual_address + offset, buf.len());

            //debugln!("Read {:X} mapped from {:X} to {:X} offset {} length {} size {} result {:?}", physical_address, buf.as_ptr() as usize, virtual_address + offset, offset, buf.len(), virtual_size, result);

            self.release(virtual_address);

            result
        } else {
            debugln!("{}:{} fault {:X} {}", file!(), line!(), buf.as_ptr() as usize, buf.len());
            Err(Error::new(EFAULT))
        }
    }

    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let contexts = unsafe { & *::env().contexts.get() };
        let current = try!(contexts.current());
        if let Ok(physical_address) = current.translate(buf.as_ptr() as usize, buf.len()) {
            let offset = physical_address % 4096;

            let virtual_address = try!(self.capture(physical_address - offset, buf.len() + offset, false));

            let result = self.call(SYS_WRITE, self.file_id, virtual_address + offset, buf.len());

            // debugln!("Write {:X} mapped from {:X} to {:X} offset {} length {} result {:?}", physical_address, buf.as_ptr() as usize, virtual_address + offset, offset, buf.len(), result);

            self.release(virtual_address);

            result
        } else {
            debugln!("{}:{} fault {:X} {}", file!(), line!(), buf.as_ptr() as usize, buf.len());
            Err(Error::new(EFAULT))
        }
    }

    /// Seek
    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        let (whence, offset) = match pos {
            ResourceSeek::Start(offset) => (SEEK_SET, offset as usize),
            ResourceSeek::Current(offset) => (SEEK_CUR, offset as usize),
            ResourceSeek::End(offset) => (SEEK_END, offset as usize)
        };

        self.call(SYS_LSEEK, self.file_id, offset, whence)
    }

    /// Stat
    fn stat(&self, stat: &mut Stat) -> Result<usize> {
        let buf = unsafe { slice::from_raw_parts_mut(stat as *mut Stat as *mut u8, size_of::<Stat>()) };

        let contexts = unsafe { & *::env().contexts.get() };
        let current = try!(contexts.current());
        if let Ok(physical_address) = current.translate(buf.as_mut_ptr() as usize, buf.len()) {
            let offset = physical_address % 4096;

            let virtual_address = try!(self.capture(physical_address - offset, buf.len() + offset, true));

            let result = self.call(SYS_FSTAT, self.file_id, virtual_address + offset, buf.len());

            self.release(virtual_address);

            result
        } else {
            debugln!("{}:{} fault {:X} {}", file!(), line!(), buf.as_ptr() as usize, buf.len());
            Err(Error::new(EFAULT))
        }
    }

    /// Sync the resource
    fn sync(&mut self) -> Result<()> {
        self.call(SYS_FSYNC, self.file_id, 0, 0).and(Ok(()))
    }

    fn truncate(&mut self, len: usize) -> Result<()> {
        self.call(SYS_FTRUNCATE, self.file_id, len, 0).and(Ok(()))
    }
}

impl Drop for SchemeResource {
    fn drop(&mut self) {
        let _ = self.call(SYS_CLOSE, self.file_id, 0, 0);
    }
}

pub struct SchemeServerResource {
    inner: Arc<SchemeInner>,
}

impl Resource for SchemeServerResource {
    /// Duplicate the resource
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box SchemeServerResource {
            inner: self.inner.clone()
        })
    }

    /// Return the url of this resource
    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;

        let path_a = b":";
        while i < buf.len() && i < path_a.len() {
            buf[i] = path_a[i];
            i += 1;
        }

        let path_b = self.inner.name.as_bytes();
        while i < buf.len() && i - path_a.len() < path_b.len() {
            buf[i] = path_b[i - path_a.len()];
            i += 1;
        }

        Ok(i)
    }


    /// Read data to buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.len() >= size_of::<Packet>() {
            let mut i = 0;

            let packet = self.inner.todo.receive();
            unsafe { ptr::write(buf.as_mut_ptr().offset(i as isize) as *mut Packet, packet); }
            i += size_of::<Packet>();

            while i + size_of::<Packet>() <= buf.len() {
                if let Some(packet) = unsafe { self.inner.todo.inner() }.pop_front() {
                    unsafe { ptr::write(buf.as_mut_ptr().offset(i as isize) as *mut Packet, packet); }
                    i += size_of::<Packet>();
                } else {
                    break;
                }
            }

            Ok(i)
        } else {
            Err(Error::new(EINVAL))
        }
    }

    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if buf.len() >= size_of::<Packet>() {
            let mut i = 0;

            while i <= buf.len() - size_of::<Packet>() {
                let packet = unsafe { & *(buf.as_ptr().offset(i as isize) as *const Packet) };
                self.inner.done.send(packet.id, (packet.a, packet.b, packet.c, packet.d));
                i += size_of::<Packet>();
            }

            Ok(i)
        } else {
            Err(Error::new(EINVAL))
        }
    }

    /// Seek
    fn seek(&mut self, _pos: ResourceSeek) -> Result<usize> {
        Err(Error::new(ESPIPE))
    }

    /// Sync the resource
    fn sync(&mut self) -> Result<()> {
        Err(Error::new(EINVAL))
    }

    fn truncate(&mut self, _len: usize) -> Result<()> {
        Err(Error::new(EINVAL))
    }
}

/// Scheme has to be wrapped
pub struct Scheme {
    name: String,
    inner: Weak<SchemeInner>
}

impl Scheme {
    pub fn new(name: &str) -> Result<(Box<Scheme>, Box<Resource>)> {
        let contexts = unsafe { &mut *::env().contexts.get() };
        let mut current = try!(contexts.current_mut());
        let server = box SchemeServerResource {
            inner: Arc::new(SchemeInner::new(name, current.deref_mut()))
        };
        let scheme = box Scheme {
            name: name.to_owned(),
            inner: Arc::downgrade(&server.inner)
        };
        Ok((scheme, server))
    }

    fn call(&self, a: usize, b: usize, c: usize, d: usize) -> Result<usize> {
        SchemeInner::call(&self.inner, a, b, c, d)
    }

    fn capture(&self, physical_address: usize, size: usize, writeable: bool) -> Result<usize> {
        SchemeInner::capture(&self.inner, physical_address, size, writeable)
    }

    fn release(&self, virtual_address: usize){
        SchemeInner::release(&self.inner, virtual_address);
    }
}

impl KScheme for Scheme {
    fn on_irq(&mut self, _irq: u8) {

    }

    fn scheme(&self) -> &str {
        &self.name
    }

    fn open(&mut self, url: Url, flags: usize) -> Result<Box<Resource>> {
        let c_str = url.to_string() + "\0";

        let virtual_address = try!(self.capture(c_str.as_ptr() as usize, c_str.len(), false));

        let result = self.call(SYS_OPEN, virtual_address, flags, 0);

        self.release(virtual_address);

        match result {
            Ok(file_id) => Ok(box SchemeResource {
                inner: self.inner.clone(),
                file_id: file_id,
            }),
            Err(err) => Err(err)
        }
    }

    fn mkdir(&mut self, url: Url, flags: usize) -> Result<()> {
        let c_str = url.to_string() + "\0";

        let virtual_address = try!(self.capture(c_str.as_ptr() as usize, c_str.len(), false));

        let result = self.call(SYS_MKDIR, virtual_address, flags, 0);

        self.release(virtual_address);

        result.and(Ok(()))
    }

    fn rmdir(&mut self, url: Url) -> Result<()> {
        let c_str = url.to_string() + "\0";

        let virtual_address = try!(self.capture(c_str.as_ptr() as usize, c_str.len(), false));

        let result = self.call(SYS_RMDIR, virtual_address, 0, 0);

        self.release(virtual_address);

        result.and(Ok(()))
    }

    fn stat(&mut self, url: Url, stat: &mut Stat) -> Result<()> {
        let buf = unsafe { slice::from_raw_parts_mut(stat as *mut Stat as *mut u8, size_of::<Stat>()) };

        let contexts = unsafe { & *::env().contexts.get() };
        let current = try!(contexts.current());
        if let Ok(physical_address) = current.translate(buf.as_mut_ptr() as usize, buf.len()) {
            let offset = physical_address % 4096;

            let virtual_address = try!(self.capture(physical_address - offset, buf.len() + offset, true));

            let c_str = url.to_string() + "\0";

            let c_str_address = try!(self.capture(c_str.as_ptr() as usize, c_str.len(), false));

            let result = self.call(SYS_STAT, c_str_address, virtual_address + offset, buf.len());

            self.release(c_str_address);

            result.and(Ok(()))
        } else {
            debugln!("{}:{} fault {:X} {}", file!(), line!(), buf.as_ptr() as usize, buf.len());
            Err(Error::new(EFAULT))
        }
    }

    fn unlink(&mut self, url: Url) -> Result<()> {
        let c_str = url.to_string() + "\0";

        let virtual_address = try!(self.capture(c_str.as_ptr() as usize, c_str.len(), false));

        let result = self.call(SYS_UNLINK, virtual_address, 0, 0);

        self.release(virtual_address);

        result.and(Ok(()))
    }
}
