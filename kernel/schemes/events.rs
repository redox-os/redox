use core::cmp;
use common::string::{String, ToString};
use common::events::Event;
use schemes::{KScheme, Resource, ResourceSeek, URL};

pub struct EventScheme;

pub struct EventResource {
    pub queue: Queue<Event>;
}

impl Resource for EventResource {
    fn url(&self) -> URL {
        return URL::from_string(&("events://".to_string()));
    }

    fn add_event(&mut self, event: Event) {
        self.queue.push(event);
    }

    // might make more sense to just return 1 event at a time
    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        let evts_to_read = cmp::min(buf.len() / 16, queue.len());
        if evts_to_read == 0 { return None; }
        let mut i = 0;
        while i < evts_to_read {
            let evt: [u32; 4] = queue.pop().data;
            let bptr = &mut buf[i*16] as *mut _ as *mut [u32;4];
            unsafe { *bptr = evt; }
            i += 1;
        }
        Some(i*16)
    }
}

// TODO: make a URL for keyboard events and one for mouse events?
impl KScheme for EventScheme {
    fn scheme(&self) -> String {
        return "events";
    }

    // TODO: make a URL for keyboard events and one for mouse events?
    fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
        unsafe {
            return Some(box EventResource {
                queue: Queue<Event>::new(),
            });
        }
    }

}
