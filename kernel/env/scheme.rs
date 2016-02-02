use alloc::arc::{Arc, Weak};
use alloc::boxed::Box;

use collections::{BTreeMap, String};
use collections::string::ToString;

use core::cell::Cell;
use core::mem::size_of;
use core::ops::DerefMut;

use arch::context::{context_switch, Context, ContextMemory};

use schemes::{Result, Resource, ResourceSeek, KScheme, Url};

use sync::Intex;

use system::error::{Error, EBADF, EFAULT, EINVAL, ESPIPE, ESRCH};
use system::scheme::Packet;
use system::syscall::{SYS_CLOSE, SYS_FSYNC, SYS_FTRUNCATE,
                    SYS_LSEEK, SEEK_SET, SEEK_CUR, SEEK_END,
                    SYS_OPEN, SYS_READ, SYS_WRITE, SYS_UNLINK};

struct SchemeInner<'a> {
    name: &'a str,
    context: *mut Context<'a>,
    next_id: Cell<usize>,
    todo: Intex<BTreeMap<usize, (usize, usize, usize, usize)>>,
    done: Intex<BTreeMap<usize, (usize, usize, usize, usize)>>,
}

impl<'a> SchemeInner<'a> {
    fn new(name: &'a str, context: *mut Context<'a>) -> SchemeInner<'a> {
        SchemeInner {
            name: name,
            context: context,
            next_id: Cell::new(1),
            todo: Intex::new(BTreeMap::new()),
            done: Intex::new(BTreeMap::new()),
        }
    }

    fn call(inner: &Weak<SchemeInner>, a: usize, b: usize, c: usize, d: usize) -> Result<usize> {
        let id;
        if let Some(scheme) = inner.upgrade() {
            id = scheme.next_id.get();

            //TODO: What should be done about collisions in self.todo or self.done?
            let mut next_id = id + 1;
            if next_id <= 0 {
                next_id = 1;
            }
            scheme.next_id.set(next_id);

            scheme.todo.lock().insert(id, (a, b, c, d));
        } else {
            return Err(Error::new(EBADF));
        }

        loop {
            if let Some(scheme) = inner.upgrade() {
                if let Some(regs) = scheme.done.lock().remove(&id) {
                    return Error::demux(regs.0);
                }
            } else {
                return Err(Error::new(EBADF));
            }

            unsafe { context_switch(false) } ;
        }
    }
}

impl<'a> Drop for SchemeInner<'a> {
    fn drop(&mut self) {
        let mut schemes = ::env().schemes.lock();

        let mut i = 0;
        while i < schemes.len() {
            let mut remove = false;
            if let Some(scheme) = schemes.get(i){
                if scheme.scheme() == self.name {
                    remove = true;
                }
            }

            if remove {
                schemes.remove(i);
            } else {
                i += 1;
            }
        }
    }
}

pub struct SchemeResource<'a> {
    inner: Weak<SchemeInner<'a>>,
    file_id: usize,
}

impl<'a> SchemeResource<'a> {
    fn call(&self, a: usize, b: usize, c: usize, d: usize) -> Result<usize> {
        SchemeInner::call(&self.inner, a, b, c, d)
    }
}

impl<'a> Resource for SchemeResource<'a> {
    /// Duplicate the resource
    fn dup(&self) -> Result<Box<Resource>> {
        Err(Error::new(EBADF))
    }

    /// Return the url of this resource
    fn url(&self) -> Url {
        Url::new()
    }

    /// Read data to buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let contexts = ::env().contexts.lock();
        if let Some(current) = contexts.current() {
            if let Some(physical_address) = unsafe { current.translate(buf.as_mut_ptr() as usize) } {
                let offset = physical_address % 4096;

                let mut virtual_address = 0;
                let virtual_size = (buf.len() + offset + 4095)/4096 * 4096;
                if let Some(scheme) = self.inner.upgrade() {
                    unsafe {
                        virtual_address = (*scheme.context).next_mem();
                        (*(*scheme.context).memory.get()).push(ContextMemory {
                            physical_address: physical_address - offset,
                            virtual_address: virtual_address,
                            virtual_size: virtual_size,
                            writeable: true,
                            allocated: false,
                        });
                    }
                }

                if virtual_address > 0 {
                    let result = self.call(SYS_READ, self.file_id, virtual_address + offset, buf.len());

                    //debugln!("Read {:X} mapped from {:X} to {:X} offset {} length {} size {} result {:?}", physical_address, buf.as_ptr() as usize, virtual_address + offset, offset, buf.len(), virtual_size, result);

                    if let Some(scheme) = self.inner.upgrade() {
                        unsafe {
                            if let Some(mut mem) = (*scheme.context).get_mem_mut(virtual_address) {
                                mem.virtual_size = 0;
                            }
                            (*scheme.context).clean_mem();
                        }
                    }

                    result
                } else {
                    Err(Error::new(EBADF))
                }
            } else {
                Err(Error::new(EFAULT))
            }
        } else {
            Err(Error::new(ESRCH))
        }
    }

    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let contexts = ::env().contexts.lock();
        if let Some(current) = contexts.current() {
            if let Some(physical_address) = unsafe { current.translate(buf.as_ptr() as usize) } {
                let offset = physical_address % 4096;

                let mut virtual_address = 0;
                let virtual_size = (buf.len() + offset + 4095)/4096 * 4096;
                if let Some(scheme) = self.inner.upgrade() {
                    unsafe {
                        virtual_address = (*scheme.context).next_mem();
                        (*(*scheme.context).memory.get()).push(ContextMemory {
                            physical_address: physical_address - offset,
                            virtual_address: virtual_address,
                            virtual_size: virtual_size,
                            writeable: true,
                            allocated: false,
                        });
                    }
                }

                if virtual_address > 0 {
                    let result = self.call(SYS_WRITE, self.file_id, virtual_address + offset, buf.len());

                    //debugln!("Write {:X} mapped from {:X} to {:X} offset {} length {} size {} result {:?}", physical_address, buf.as_ptr() as usize, virtual_address + offset, offset, buf.len(), virtual_size, result);

                    if let Some(scheme) = self.inner.upgrade() {
                        unsafe {
                            if let Some(mut mem) = (*scheme.context).get_mem_mut(virtual_address) {
                                mem.virtual_size = 0;
                            }
                            (*scheme.context).clean_mem();
                        }
                    }

                    result
                } else {
                    Err(Error::new(EBADF))
                }
            } else {
                Err(Error::new(EFAULT))
            }
        } else {
            Err(Error::new(ESRCH))
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

    /// Sync the resource
    fn sync(&mut self) -> Result<()> {
        self.call(SYS_FSYNC, self.file_id, 0, 0).and(Ok(()))
    }

    fn truncate(&mut self, len: usize) -> Result<()> {
        self.call(SYS_FTRUNCATE, self.file_id, len, 0).and(Ok(()))
    }
}

impl<'a> Drop for SchemeResource<'a> {
    fn drop(&mut self) {
        let _ = self.call(SYS_CLOSE, self.file_id, 0, 0);
    }
}

pub struct SchemeServerResource<'a> {
    inner: Arc<SchemeInner<'a>>,
}

impl<'a> Resource for SchemeServerResource<'a> {
    /// Duplicate the resource
    fn dup<'b>(&'b self) -> Result<Box<Resource + 'b>> {
        Ok(box SchemeServerResource {
            inner: self.inner.clone()
        })
    }

    /// Return the url of this resource
    fn url(&self) -> Url {
        Url::from_string(":".to_string() + &self.inner.name)
    }

    /// Read data to buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.len() == size_of::<Packet>() {
            let packet_ptr: *mut Packet = buf.as_mut_ptr() as *mut Packet;
            let packet = unsafe { &mut *packet_ptr };

            let mut todo = self.inner.todo.lock();

            packet.id = if let Some(id) = todo.keys().next() {
                *id
            } else {
                0
            };

            if packet.id > 0 {
                if let Some(regs) = todo.remove(&packet.id) {
                    packet.a = regs.0;
                    packet.b = regs.1;
                    packet.c = regs.2;
                    packet.d = regs.3;
                    return Ok(size_of::<Packet>())
                }
            }

            Ok(0)
        } else {
            Err(Error::new(EINVAL))
        }
    }

    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if buf.len() == size_of::<Packet>() {
            let packet_ptr: *const Packet = buf.as_ptr() as *const Packet;
            let packet = unsafe { & *packet_ptr };
            self.inner.done.lock().insert(packet.id, (packet.a, packet.b, packet.c, packet.d));
            Ok(size_of::<Packet>())
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
pub struct Scheme<'a> {
    name: &'a str,
    inner: Weak<SchemeInner<'a>>,
}

impl<'a> Scheme<'a> {
    pub fn new(name: &'a str) -> Result<(Scheme<'a>, Box<Resource + 'a>)> {
        if let Some(context) = ::env().contexts.lock().current_mut() {
            let server = box SchemeServerResource {
                inner: Arc::new(SchemeInner::new(&name.to_string(), context.deref_mut()))
            };
            let scheme = Scheme {
                name: name,
                inner: Arc::downgrade(&server.inner)
            };
            Ok((scheme, server))
        } else {
            Err(Error::new(ESRCH))
        }
    }

    fn call(&self, a: usize, b: usize, c: usize, d: usize) -> Result<usize> {
        SchemeInner::call(&self.inner, a, b, c, d)
    }
}

impl<'a> KScheme for Scheme<'a> {
    fn on_irq(&mut self, _irq: u8) {

    }

    fn on_poll(&mut self) {

    }

    fn scheme(&self) -> &str {
        &self.name
    }

    fn open<'b>(&'b mut self, url: &Url, flags: usize) -> Result<Box<Resource + 'b>> {
        let c_str = url.string.clone() + "\0";

        let physical_address = c_str.as_ptr() as usize;

        let mut virtual_address = 0;
        if let Some(scheme) = self.inner.upgrade() {
            unsafe {
                virtual_address = (*scheme.context).next_mem();
                (*(*scheme.context).memory.get()).push(ContextMemory {
                    physical_address: physical_address,
                    virtual_address: virtual_address,
                    virtual_size: c_str.len(),
                    writeable: false,
                    allocated: false,
                });
            }
        }

        if virtual_address > 0 {
            let result = self.call(SYS_OPEN, virtual_address, flags, 0);

            if let Some(scheme) = self.inner.upgrade() {
                unsafe {
                    if let Some(mut mem) = (*scheme.context).get_mem_mut(virtual_address) {
                        mem.virtual_size = 0;
                    }
                    (*scheme.context).clean_mem();
                }
            }

            match result {
                Ok(file_id) => Ok(box SchemeResource {
                    inner: self.inner.clone(),
                    file_id: file_id,
                }),
                Err(err) => Err(err)
            }
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn unlink(&mut self, url: &Url) -> Result<()> {
        let c_str = url.string.clone() + "\0";

        let physical_address = c_str.as_ptr() as usize;

        let mut virtual_address = 0;
        if let Some(scheme) = self.inner.upgrade() {
            unsafe {
                virtual_address = (*scheme.context).next_mem();
                (*(*scheme.context).memory.get()).push(ContextMemory {
                    physical_address: physical_address,
                    virtual_address: virtual_address,
                    virtual_size: c_str.len(),
                    writeable: false,
                    allocated: false,
                });
            }
        }

        if virtual_address > 0 {
            let result = self.call(SYS_UNLINK, virtual_address, 0, 0);

            if let Some(scheme) = self.inner.upgrade() {
                unsafe {
                    if let Some(mut mem) = (*scheme.context).get_mem_mut(virtual_address) {
                        mem.virtual_size = 0;
                    }
                    (*scheme.context).clean_mem();
                }
            }

            result.and(Ok(()))
        } else {
            Err(Error::new(EBADF))
        }
    }
}
