use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;

use core::cell::UnsafeCell;

use arch::context::ContextManager;
use common::event::Event;
use common::time::Duration;
use disk::Disk;
use network::Nic;
use fs::{KScheme, Resource, Scheme, VecResource, Url};
use sync::WaitQueue;

use system::error::{Error, Result, ENOENT, EEXIST};
use system::syscall::{O_CREAT, Stat};

use self::console::Console;
use self::log::Log;

/// The Kernel Console
pub mod console;

/// The Kernel Log
pub mod log;

/// The kernel environment
pub struct Environment {
    /// Contexts
    pub contexts: UnsafeCell<ContextManager>,

    /// Clock realtime (default)
    pub clock_realtime: UnsafeCell<Duration>,
    /// Monotonic clock
    pub clock_monotonic: UnsafeCell<Duration>,

    /// Default console
    pub console: UnsafeCell<Console>,
    /// Disks
    pub disks: UnsafeCell<Vec<Box<Disk>>>,
    /// Network interfaces
    pub nics: UnsafeCell<Vec<Box<Nic>>>,
    /// Pending events
    pub events: WaitQueue<Event>,
    /// Kernel logs
    pub log: UnsafeCell<Log>,
    /// Schemes
    pub schemes: UnsafeCell<Vec<Box<KScheme>>>,

    /// Interrupt stats
    pub interrupts: UnsafeCell<[u64; 256]>,
}

impl Environment {
    pub fn new() -> Box<Environment> {
        box Environment {
            contexts: UnsafeCell::new(ContextManager::new()),

            clock_realtime: UnsafeCell::new(Duration::new(0, 0)),
            clock_monotonic: UnsafeCell::new(Duration::new(0, 0)),

            console: UnsafeCell::new(Console::new()),
            disks: UnsafeCell::new(Vec::new()),
            nics: UnsafeCell::new(Vec::new()),
            events: WaitQueue::new(),
            log: UnsafeCell::new(Log::new()),
            schemes: UnsafeCell::new(Vec::new()),

            interrupts: UnsafeCell::new([0; 256]),
        }
    }

    pub fn on_irq(&self, irq: u8) {
        for mut scheme in unsafe { &mut *self.schemes.get() }.iter_mut() {
            scheme.on_irq(irq);
        }
    }

    /// Open a new resource
    pub fn open(&self, url: Url, flags: usize) -> Result<Box<Resource>> {
        let url_scheme = url.scheme();
        if url_scheme.is_empty() {
            let url_path = url.reference();
            if url_path.trim_matches('/').is_empty() {
                let mut list = String::new();

                for scheme in unsafe { &mut *self.schemes.get() }.iter() {
                    let scheme_str = scheme.scheme();
                    if !scheme_str.is_empty() {
                        if !list.is_empty() {
                            list = list + "\n" + scheme_str;
                        } else {
                            list = scheme_str.to_string();
                        }
                    }
                }

                Ok(box VecResource::new(":".to_string(), list.into_bytes()))
            } else if flags & O_CREAT == O_CREAT {
                for scheme in unsafe { &mut *self.schemes.get() }.iter_mut() {
                    if scheme.scheme() == url_path {
                        return Err(Error::new(EEXIST));
                    }
                }

                match Scheme::new(url_path) {
                    Ok((scheme, server)) => {
                        unsafe { &mut *self.schemes.get() }.push(scheme);
                        Ok(server)
                    },
                    Err(err) => Err(err)
                }
            } else {
                Err(Error::new(ENOENT))
            }
        } else {
            for mut scheme in unsafe { &mut *self.schemes.get() }.iter_mut() {
                if scheme.scheme() == url_scheme {
                    return scheme.open(url, flags);
                }
            }
            Err(Error::new(ENOENT))
        }
    }

    /// Makes a directory
    pub fn mkdir(&self, url: Url, flags: usize) -> Result<()> {
        let url_scheme = url.scheme();
        if !url_scheme.is_empty() {
            for mut scheme in unsafe { &mut *self.schemes.get() }.iter_mut() {
                if scheme.scheme() == url_scheme {
                    return scheme.mkdir(url, flags);
                }
            }
        }
        Err(Error::new(ENOENT))
    }

    /// Remove a directory
    pub fn rmdir(&self, url: Url) -> Result<()> {
        let url_scheme = url.scheme();
        if !url_scheme.is_empty() {
            for mut scheme in unsafe { &mut *self.schemes.get() }.iter_mut() {
                if scheme.scheme() == url_scheme {
                    return scheme.rmdir(url);
                }
            }
        }
        Err(Error::new(ENOENT))
    }

    /// Stat a path
    pub fn stat(&self, url: Url, stat: &mut Stat) -> Result<()> {
        let url_scheme = url.scheme();
        if !url_scheme.is_empty() {
            for mut scheme in unsafe { &mut *self.schemes.get() }.iter_mut() {
                if scheme.scheme() == url_scheme {
                    return scheme.stat(url, stat);
                }
            }
        }
        Err(Error::new(ENOENT))
    }

    /// Unlink a resource
    pub fn unlink(&self, url: Url) -> Result<()> {
        let url_scheme = url.scheme();
        if !url_scheme.is_empty() {
            for mut scheme in unsafe { &mut *self.schemes.get() }.iter_mut() {
                if scheme.scheme() == url_scheme {
                    return scheme.unlink(url);
                }
            }
        }
        Err(Error::new(ENOENT))
    }
}
