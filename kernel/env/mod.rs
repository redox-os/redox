use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;
use collections::vec_deque::VecDeque;

use common::event::Event;
use sync::Intex;
use sync::Mutex;
use common::time::Duration;

use core::cell::UnsafeCell;

use scheduler::context::ContextManager;

use schemes::{Result, KScheme, Resource, VecResource, Url};

use syscall::{SysError, ENOENT};

use self::console::Console;

/// The Kernel Console
pub mod console;

/// The kernel environment
pub struct Environment {
    /// Contexts
    pub contexts: Intex<ContextManager>,

    /// Clock realtime (default)
    pub clock_realtime: Intex<Duration>,
    /// Monotonic clock
    pub clock_monotonic: Intex<Duration>,

    /// Default console
    pub console: Intex<Console>,
    /// Pending events
    pub events: Mutex<VecDeque<Event>>,
    /// Schemes
    pub schemes: Vec<UnsafeCell<Box<KScheme>>>,

    /// Interrupt stats
    pub interrupts: Intex<[u64; 256]>
}

impl Environment {
    pub fn new() -> Box<Environment> {
        box Environment {
            contexts: Intex::new(ContextManager::new()),

            clock_realtime: Intex::new(Duration::new(0, 0)),
            clock_monotonic: Intex::new(Duration::new(0, 0)),

            console: Intex::new(Console::new()),
            events: Mutex::new(VecDeque::new()),
            schemes: Vec::new(),

            interrupts: Intex::new([0; 256])
        }
    }

    pub fn on_irq(&self, irq: u8) {
        for scheme in self.schemes.iter() {
            unsafe { (*scheme.get()).on_irq(irq) };
        }
    }

    pub fn on_poll(&self) {
        for scheme in self.schemes.iter() {
            unsafe { (*scheme.get()).on_poll() };
        }
    }

    /// Open a new resource
    pub fn open(&self, url: &Url, flags: usize) -> Result<Box<Resource>> {
        let url_scheme = url.scheme();
        if url_scheme.is_empty() {
            let mut list = String::new();

            for scheme in self.schemes.iter() {
                let scheme_str = unsafe { (*scheme.get()).scheme() };
                if !scheme_str.is_empty() {
                    if !list.is_empty() {
                        list = list + "\n" + scheme_str;
                    } else {
                        list = scheme_str.to_string();
                    }
                }
            }

            Ok(box VecResource::new(Url::new(), list.into_bytes()))
        } else {
            for scheme in self.schemes.iter() {
                let scheme_str = unsafe { (*scheme.get()).scheme() };
                if scheme_str == url_scheme {
                    return unsafe { (*scheme.get()).open(url, flags) };
                }
            }
            Err(SysError::new(ENOENT))
        }
    }

    /// Unlink a resource
    pub fn unlink(&self, url: &Url) -> Result<()> {
        let url_scheme = url.scheme();
        if !url_scheme.is_empty() {
            for scheme in self.schemes.iter() {
                let scheme_str = unsafe { (*scheme.get()).scheme() };
                if scheme_str == url_scheme {
                    return unsafe { (*scheme.get()).unlink(url) };
                }
            }
        }
        Err(SysError::new(ENOENT))
    }
}
