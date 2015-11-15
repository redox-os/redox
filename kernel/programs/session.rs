use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;

use scheduler;

use schemes::KScheme;
use schemes::{Resource, Url, VecResource};

/// A session
pub struct Session {
    /// The scheme items
    pub items: Vec<Box<KScheme>>,
}

impl Session {
    /// Create new session
    pub fn new() -> Box<Self> {
        box Session {
            items: Vec::new(),
        }
    }

    pub unsafe fn on_irq(&mut self, irq: u8) {
        let reenable = scheduler::start_no_ints();
        for mut item in self.items.iter_mut() {
            item.on_irq(irq);
        }
        scheduler::end_no_ints(reenable);
    }

    pub unsafe fn on_poll(&mut self) {
        let reenable = scheduler::start_no_ints();
        for mut item in self.items.iter_mut() {
            item.on_poll();
        }
        scheduler::end_no_ints(reenable);
    }

    /// Open a new resource
    pub fn open(&mut self, url: &Url, flags: usize) -> Option<Box<Resource>> {
        if url.scheme().len() == 0 {
            let mut list = String::new();

            for item in self.items.iter() {
                let scheme = item.scheme();
                if !scheme.is_empty() {
                    if !list.is_empty() {
                        list = list + "\n" + scheme;
                    } else {
                        list = scheme.to_string();
                    }
                }
            }

            Some(box VecResource::new(Url::new(), list.into_bytes()))
        } else {
            for mut item in self.items.iter_mut() {
                if item.scheme() == url.scheme() {
                    return item.open(url, flags);
                }
            }
            None
        }
    }
}
