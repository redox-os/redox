use super::package::*;
use super::executor::*;

use alloc::boxed::Box;

use core::cmp;

use common::event::{self, Event, EventOption, KeyEvent, MouseEvent};
use common::scheduler;
use common::string::{String, ToString};
use common::vec::Vec;

use graphics::point::Point;
use graphics::size::Size;
use graphics::window::Window;

use schemes::KScheme;
use schemes::{Resource, URL, VecResource};

pub struct Session {
    pub items: Vec<Box<KScheme>>,
}

impl Session {
    pub fn new() -> Box<Self> {
        unsafe {
            box Session {
                items: Vec::new(),
            }
        }
    }

    pub unsafe fn on_irq(&mut self, irq: u8) {
        for item in self.items.iter() {
            let reenable = scheduler::start_no_ints();
            item.on_irq(irq);
            scheduler::end_no_ints(reenable);
        }
    }

    pub unsafe fn on_poll(&mut self) {
        for item in self.items.iter() {
            let reenable = scheduler::start_no_ints();
            item.on_poll();
            scheduler::end_no_ints(reenable);
        }
    }

    pub fn open(&self, url: &URL) -> Option<Box<Resource>> {
        if url.scheme().len() == 0 {
            let mut list = String::new();

            for item in self.items.iter() {
                let scheme = item.scheme();
                if scheme.len() > 0 {
                    if list.len() > 0 {
                        list = list + "\n" + scheme;
                    } else {
                        list = scheme;
                    }
                }
            }

            Some(box VecResource::new(URL::new(), list.to_utf8()))
        } else {
            for item in self.items.iter() {
                if item.scheme() == url.scheme() {
                    return item.open(url);
                }
            }
            None
        }
    }
}
