use alloc::boxed::Box;

use scheduler::context::context_switch;
use scheduler;

use schemes::{KScheme, Resource, URL};

use syscall::handle;

/// A debug resource
pub struct DebugResource;

impl Resource for DebugResource {
    fn dup(&self) -> Option<Box<Resource>> {
        Some(box DebugResource)
    }

    fn url(&self) -> URL {
        return URL::from_str("debug://");
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        unsafe {
            loop {
                let reenable = scheduler::start_no_ints();

                if !(*::debug_command).is_empty() {
                    break;
                }

                scheduler::end_no_ints(reenable);

                context_switch(false);
            }

            let reenable = scheduler::start_no_ints();

            //TODO: Unicode
            let mut i = 0;
            while i < buf.len() && !(*::debug_command).as_mut_vec().is_empty() {
                buf[i] = (*::debug_command).as_mut_vec().remove(0);
                i += 1;
            }

            scheduler::end_no_ints(reenable);

            return Some(i);
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        for byte in buf {
            unsafe {
                handle::do_sys_debug(*byte);
            }
        }
        return Some(buf.len());
    }

    fn sync(&mut self) -> bool {
        true
    }
}

pub struct DebugScheme;

impl KScheme for DebugScheme {
    fn scheme(&self) -> &str {
        "debug"
    }

    fn open(&mut self, _: &URL) -> Option<Box<Resource>> {
        Some(box DebugResource)
    }
}
