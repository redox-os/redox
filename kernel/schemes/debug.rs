use alloc::boxed::Box;

use scheduler::context::context_switch;
use scheduler;

use schemes::{KScheme, Resource, Url};

use syscall::handle;

/// A debug resource
pub struct DebugResource {
    pub line_toggle: bool,
}

impl Resource for DebugResource {
    fn dup(&self) -> Option<Box<Resource>> {
        Some(box DebugResource {
            line_toggle: self.line_toggle
        })
    }

    fn url(&self) -> Url {
        return Url::from_str("debug://");
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        if self.line_toggle {
            self.line_toggle = false;
            Some(0)
        }else{
            unsafe {
                loop {
                    let reenable = scheduler::start_no_ints();

                    if !(*::debug_command).is_empty() {
                        break;
                    }

                    scheduler::end_no_ints(reenable);

                    let disable = scheduler::start_ints();
                    scheduler::end_ints(disable);
                }

                let reenable = scheduler::start_no_ints();

                //TODO: Unicode
                let mut i = 0;
                while i < buf.len() && !(*::debug_command).as_mut_vec().is_empty() {
                    buf[i] = (*::debug_command).as_mut_vec().remove(0);
                    i += 1;
                }

                if i > 0 && (*::debug_command).is_empty() {
                    self.line_toggle = true;
                }

                scheduler::end_no_ints(reenable);

                Some(i)
            }
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

    fn open(&mut self, _: &Url, _: usize) -> Option<Box<Resource>> {
        Some(box DebugResource {
            line_toggle: false
        })
    }
}
