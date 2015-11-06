use alloc::boxed::Box;
use core::{cmp, mem, ptr};
use collections::string::*;
use common::event::{Event};
use common::queue::Queue;
use scheduler;
use schemes::{KScheme, Resource, ResourceSeek, Url};

use common::debug;
static mut events_to_userspace_ptr: *mut Queue<Event> = 0 as *mut Queue<Event>;

pub struct EventScheme;

pub struct EventResource;

impl EventResource {
    pub fn add_event(event: Event) {
        unsafe {
            let events = &mut *events_to_userspace_ptr;
            let reenable = scheduler::start_no_ints();
            events.push(event);
            scheduler::end_no_ints(reenable);
        }
    }
}
impl Resource for EventResource {
    /// Return the Url for event resource
    fn url(&self) -> Url {
        return Url::from_string("events://".to_string());
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        let data_size = mem::size_of::<Event>();
        if buf.len() < data_size {
            Some(0)
        } else {
            unsafe {
                let events = &mut *events_to_userspace_ptr;
                let reenable = scheduler::start_no_ints();
                let event = events.pop();
                scheduler::end_no_ints(reenable);
                match event {
                    Some(evt) => {
                        ptr::write(buf.as_ptr() as *mut Event, evt);
                        Some(data_size)
                    },
                    None => None
                }
            }
        }
    }
}

impl KScheme for EventScheme {
    fn scheme(&self) -> &str {
        "events"
    }

    fn open(&mut self, url: &Url, _: usize) -> Option<Box<Resource>> {
        unsafe {
            return Some(box EventResource);
        }
    }
}

impl EventScheme {
    pub fn init() {
        unsafe {
            events_to_userspace_ptr = Box::into_raw(box Queue::new());
        }
    }
}
