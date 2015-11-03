use super::package::*;
use super::executor::*;

use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;

use common::event::{Event, EventOption, KeyEvent, MouseEvent};
use common::scheduler;

use graphics::point::Point;
use graphics::size::Size;

use schemes::KScheme;
use schemes::{Resource, URL, VecResource};

/// A session
pub struct Session {
    pub items: Vec<Box<KScheme>>, // FIXME: Vec<Box<T>> is equiv to Vec<T>
    pub packages: Vec<Box<Package>>,
}

impl Session {
    pub fn new() -> Box<Self> {     
        unsafe {
            box Session {
                items: Vec::new(),
                packages: Vec::new(),
            }
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
    pub fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
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

            Some(box VecResource::new(URL::new(), list.into_bytes()))
        } else {
            for mut item in self.items.iter_mut() {
                if item.scheme() == url.scheme() {
                    return item.open(url);
                }
            }
            None
        }
    }
}
