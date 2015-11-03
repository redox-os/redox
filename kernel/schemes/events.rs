use alloc::boxed::Box;
use core::{cmp, mem, ptr};
use collections::string::*;
use common::event::{Event, EventData};
use common::queue::Queue;
use common::scheduler;
use schemes::{KScheme, Resource, ResourceSeek, URL};

static mut events_to_userspace_ptr: *mut Queue<EventData> = 0 as *mut Queue<EventData>;

pub struct EventScheme;

pub struct EventResource;

impl EventResource {
    pub fn add_event(event: Event) {
        unsafe {
            let events = &mut *events_to_userspace_ptr;
            let reenable = scheduler::start_no_ints();
            events.push(event.data);
            scheduler::end_no_ints(reenable);
        }
    }
}
impl Resource for EventResource {
    fn dup(&self) -> Option<Box<Resource>> {
        None
    }

    /// Return the URL for event resource
    fn url(&self) -> URL {
        return URL::from_string(&("events://".to_string()));
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        let data_size = mem::size_of::<EventData>();
        //TODO: why does this only work when length check is commented out?
        // LazyOxen
        /*
        if buf.len() < data_size {
            Some(0)
        } else {
        */
            unsafe {
                let events = &mut *events_to_userspace_ptr;
                let reenable = scheduler::start_no_ints();
                let event = events.pop();
                scheduler::end_no_ints(reenable);
                match event {
                    Some(evt) => {
                        ptr::write(buf.as_ptr() as *mut EventData, evt);
                        /*
                        let bptr = buf.as_mut_ptr() as *mut _ as *mut [isize;4];
                        *bptr = evt;
                        */
                        Some(data_size)
                    },
                    None => None
                }
            }
            /*
        }
        */
        /*
        // read multiple events?
        let mut evt_capacity = buf.len() / data_size;
        if evt_capacity == 0 { return None; }
        let mut i = 0;
        let mut j = 0;
        unsafe {
            let events = &mut *events_to_userspace_ptr;
            while i < evt_capacity {
                let reenable = scheduler::start_no_ints();
                let next_event = events.pop();
                scheduler::end_no_ints(reenable);
                match next_event {
                    Some(event) => {
                        let bptr = &mut buf[i*data_size] as *mut _ as *mut [isize;4];
                        *bptr = event;
                        j += 1;
                    },
                    None => break, // bail if we were too late to get the next event
                }
                i += 1;
            }
        }
        Some(j * data_size)
        */
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        None
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        None
    }

    fn sync(&mut self) -> bool {
        true
    }
}

impl KScheme for EventScheme {
    fn scheme(&self) -> &str {
        return "events";
    }

    fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
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
