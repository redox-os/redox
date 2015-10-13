use alloc::boxed::Box;

use common::context::context_switch;
use common::resource::{Resource, ResourceSeek, URL};
use common::scheduler;
use common::string::{String, ToString};

use programs::session::SessionItem;

use syscall::handle;

/// A debug resource
pub struct DebugResource;

impl Resource for DebugResource {
    fn dup(&self) -> Box<Resource> {
        box DebugResource
    }

    fn url(&self) -> URL {
        return URL::from_str("debug://");
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        unsafe {
            loop {
                let reenable = scheduler::start_no_ints();

                if (*::debug_command).len() > 0 {
                    break;
                }

                scheduler::end_no_ints(reenable);

                context_switch(false);
            }

            let reenable = scheduler::start_no_ints();

            //TODO: Unicode
            let mut i = 0;
            while i < buf.len() {
                match (*::debug_command).vec.remove(0) {
                    Option::Some(c) => buf[i] = c as u8,
                    Option::None => break,
                }
                i += 1;
            }

            scheduler::end_no_ints(reenable);

            return Option::Some(i);
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        for byte in buf {
            unsafe {
                handle::do_sys_debug(*byte);
            }
        }
        return Option::Some(buf.len());
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None;
    }

    fn sync(&mut self) -> bool {
        return true;
    }
}

pub struct DebugScheme;

impl SessionItem for DebugScheme {
    fn scheme(&self) -> String {
        return "debug".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        return box DebugResource;
    }
}
