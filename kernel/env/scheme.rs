use alloc::arc::{Arc, Weak};
use alloc::boxed::Box;

use collections::{BTreeMap, String};
use collections::string::ToString;

use core::cell::Cell;

use scheduler::context::context_switch;

use schemes::{Result, Resource, ResourceSeek, KScheme, Url};

use sync::Intex;

use system::error::{Error, EBADF, EINVAL, ENOENT, ESPIPE};

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

    fn call(&self, regs: &mut (usize, usize, usize, usize)) {
        let id = self.next_id.get();

        //TODO: What should be done about collisions in self.todo or self.done?
        {
            let mut next_id = id + 1;
            if next_id <= 0 {
                next_id = 1;
            }
            self.next_id.set(next_id);
        }

        self.todo.lock().insert(id, *regs);

        loop {
            if let Some(new_regs) = self.done.lock().remove(&id) {
                *regs = new_regs;
                return
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
        Err(Error::new(EBADF))
    }

    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    /// Seek
    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    /// Sync the resource
    fn sync(&mut self) -> Result<()> {
        Err(Error::new(EBADF))
    }

    fn truncate(&mut self, len: usize) -> Result<()> {
        Err(Error::new(EBADF))
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
            Ok(0)
        }else {
            Err(Error::new(EBADF))
        }
    }

    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if let Some(scheme) = self.inner.upgrade() {
            Ok(0)
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
        Err(Error::new(ENOENT))
    }

    fn unlink(&mut self, url: &Url) -> Result<()> {
        Err(Error::new(ENOENT))
    }
}
