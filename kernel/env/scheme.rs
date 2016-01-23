use alloc::arc::{Arc, Weak};
use alloc::boxed::Box;

use collections::{BTreeMap, String};
use collections::string::ToString;

use core::cell::Cell;
use core::mem::size_of;

use scheduler::context::context_switch;

use schemes::{Result, Resource, ResourceSeek, KScheme, Url};

use sync::Intex;

use system::error::{Error, EBADF, EINVAL, ENOENT, ESPIPE};
use system::scheme::Packet;
use system::syscall::{SYS_FSYNC, SYS_FTRUNCATE, SYS_LSEEK, SEEK_SET, SEEK_CUR, SEEK_END, SYS_OPEN, SYS_READ, SYS_WRITE, SYS_UNLINK};

struct SchemeInner {
    name: String,
    next_id: Cell<usize>,
    todo: Intex<BTreeMap<usize, (usize, usize, usize, usize)>>,
    done: Intex<BTreeMap<usize, (usize, usize, usize, usize)>>,
}

impl SchemeInner {
    fn new(name: String) -> SchemeInner {
        SchemeInner {
            name: name,
            next_id: Cell::new(1),
            todo: Intex::new(BTreeMap::new()),
            done: Intex::new(BTreeMap::new()),
        }
    }

    fn recv(&self, packet: &mut Packet) {
        loop {
            {
                let mut todo = self.todo.lock();

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
                        return
                    }
                }
            }

            unsafe { context_switch(false) } ;
        }
    }

    fn send(&self, packet: &Packet) {
        self.done.lock().insert(packet.id, (packet.a, packet.b, packet.c, packet.d));
    }

    fn call(&self, a: usize, b: usize, c: usize, d: usize) -> usize {
        let id = self.next_id.get();

        //TODO: What should be done about collisions in self.todo or self.done?
        {
            let mut next_id = id + 1;
            if next_id <= 0 {
                next_id = 1;
            }
            self.next_id.set(next_id);
        }

        self.todo.lock().insert(id, (a, b, c, d));

        loop {
            if let Some(regs) = self.done.lock().remove(&id) {
                return regs.0;
            }

            unsafe { context_switch(false) } ;
        }
    }
}

pub struct SchemeResource {
    inner: Weak<SchemeInner>,
    file_id: usize,
}

impl Resource for SchemeResource {
    /// Duplicate the resource
    fn dup(&self) -> Result<Box<Resource>> {
        Err(Error::new(EBADF))
    }

    /// Return the url of this resource
    fn url(&self) -> Url {
        Url::new()
    }

    // TODO: Make use of Write and Read trait
    /// Read data to buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if let Some(scheme) = self.inner.upgrade() {
            Error::demux(scheme.call(SYS_READ, self.file_id, buf.as_mut_ptr() as usize, buf.len()))
        } else {
            Err(Error::new(EBADF))
        }
    }

    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if let Some(scheme) = self.inner.upgrade() {
            Error::demux(scheme.call(SYS_WRITE, self.file_id, buf.as_ptr() as usize, buf.len()))
        } else {
            Err(Error::new(EBADF))
        }
    }

    /// Seek
    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        if let Some(scheme) = self.inner.upgrade() {
            let (whence, offset) = match pos {
                ResourceSeek::Start(offset) => (SEEK_SET, offset as usize),
                ResourceSeek::Current(offset) => (SEEK_CUR, offset as usize),
                ResourceSeek::End(offset) => (SEEK_END, offset as usize)
            };

            Error::demux(scheme.call(SYS_LSEEK, self.file_id, offset, whence))
        } else {
            Err(Error::new(EBADF))
        }
    }

    /// Sync the resource
    fn sync(&mut self) -> Result<()> {
        if let Some(scheme) = self.inner.upgrade() {
            Error::demux(scheme.call(SYS_FSYNC, self.file_id, 0, 0)).and(Ok(()))
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn truncate(&mut self, len: usize) -> Result<()> {
        if let Some(scheme) = self.inner.upgrade() {
            Error::demux(scheme.call(SYS_FTRUNCATE, self.file_id, len, 0)).and(Ok(()))
        } else {
            Err(Error::new(EBADF))
        }
    }
}

pub struct SchemeServerResource {
    inner: Weak<SchemeInner>,
}

impl Resource for SchemeServerResource {
    /// Duplicate the resource
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box SchemeServerResource {
            inner: self.inner.clone()
        })
    }

    /// Return the url of this resource
    fn url(&self) -> Url {
        if let Some(scheme) = self.inner.upgrade() {
            Url::from_string(":".to_string() + &scheme.name)
        } else {
            Url::new()
        }
    }

    // TODO: Make use of Write and Read trait
    /// Read data to buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if let Some(scheme) = self.inner.upgrade() {
            if buf.len() == size_of::<Packet>() {
                let packet_ptr: *mut Packet = buf.as_mut_ptr() as *mut Packet;
                scheme.recv(unsafe { &mut *packet_ptr });

                Ok(size_of::<Packet>())
            } else {
                Err(Error::new(EINVAL))
            }
        }else {
            Err(Error::new(EBADF))
        }
    }

    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if let Some(scheme) = self.inner.upgrade() {
            if buf.len() == size_of::<Packet>() {
                let packet_ptr: *const Packet = buf.as_ptr() as *const Packet;
                scheme.send(unsafe { &*packet_ptr });

                Ok(size_of::<Packet>())
            } else {
                Err(Error::new(EINVAL))
            }
        }else {
            Err(Error::new(EBADF))
        }
    }

    /// Seek
    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        if let Some(scheme) = self.inner.upgrade() {
            Err(Error::new(ESPIPE))
        }else {
            Err(Error::new(EBADF))
        }
    }

    /// Sync the resource
    fn sync(&mut self) -> Result<()> {
        if let Some(scheme) = self.inner.upgrade() {
            Err(Error::new(EINVAL))
        }else {
            Err(Error::new(EBADF))
        }
    }

    fn truncate(&mut self, len: usize) -> Result<()> {
        if let Some(scheme) = self.inner.upgrade() {
            Err(Error::new(EINVAL))
        }else {
            Err(Error::new(EBADF))
        }
    }
}

/// Scheme has to be wrapped
pub struct Scheme {
    inner: Arc<SchemeInner>
}

impl Scheme {
    pub fn new(name: String) -> Box<Scheme> {
        box Scheme {
            inner: Arc::new(SchemeInner::new(name))
        }
    }

    pub fn server(&self) -> Box<Resource> {
        box SchemeServerResource {
            inner: Arc::downgrade(&self.inner)
        }
    }
}

impl KScheme for Scheme {
    fn on_irq(&mut self, irq: u8) {

    }

    fn on_poll(&mut self) {

    }

    fn scheme(&self) -> &str {
        &self.inner.name
    }

    fn open(&mut self, url: &Url, flags: usize) -> Result<Box<Resource>> {
        let c_str = url.string.clone() + "\0";
        match Error::demux(self.inner.call(SYS_OPEN, c_str.as_ptr() as usize, 0, 0)) {
            Ok(file_id) => Ok(box SchemeResource {
                inner: Arc::downgrade(&self.inner),
                file_id: file_id,
            }),
            Err(err) => Err(err)
        }
    }

    fn unlink(&mut self, url: &Url) -> Result<()> {
        let c_str = url.string.clone() + "\0";
        Error::demux(self.inner.call(SYS_UNLINK, c_str.as_ptr() as usize, 0, 0)).and(Ok(()))
    }
}
