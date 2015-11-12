use alloc::boxed::Box;

use collections::string::String;

use scheduler::context::{context_switch, context_i, contexts_ptr};
use scheduler;

use schemes::{KScheme, Resource, Url};

use syscall::handle;

/// A debug resource
pub struct DebugResource {
    pub scheme: *mut DebugScheme,
    pub command: String,
    pub line_toggle: bool,
}

impl Resource for DebugResource {
    fn dup(&self) -> Option<Box<Resource>> {
        Some(box DebugResource {
            scheme: self.scheme,
            command: self.command.clone(),
            line_toggle: self.line_toggle,
        })
    }

    fn url(&self) -> Url {
        return Url::from_str("debug:");
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        if self.line_toggle {
            self.line_toggle = false;
            return Some(0);
        }

        if self.command.is_empty() {
            loop {
                unsafe {
                    let reenable = scheduler::start_no_ints();

                    // Hack!
                    if (*self.scheme).context >= (*contexts_ptr).len() ||
                       (*self.scheme).context < context_i {
                        (*self.scheme).context = context_i;
                    }

                    if (*self.scheme).context == context_i && (*::console).command.is_some() {
                        if let Some(ref command) = (*::console).command {
                            self.command = command.clone();
                        }
                        (*::console).command = None;
                        break;
                    }

                    scheduler::end_no_ints(reenable);

                    context_switch(false);
                }
            }
        }

        // TODO: Unicode
        let mut i = 0;
        while i < buf.len() && ! self.command.is_empty() {
            buf[i] = unsafe { self.command.as_mut_vec().remove(0) };
            i += 1;
        }

        if i > 0 && self.command.is_empty() {
            self.line_toggle = true;
        }

        Some(i)
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

pub struct DebugScheme {
    pub context: usize,
}

impl DebugScheme {
    pub fn new() -> Box<Self> {
        box DebugScheme { context: 0 }
    }
}

impl KScheme for DebugScheme {
    fn scheme(&self) -> &str {
        "debug"
    }

    fn open(&mut self, _: &Url, _: usize) -> Option<Box<Resource>> {
        Some(box DebugResource {
            scheme: self,
            command: String::new(),
            line_toggle: false,
        })
    }
}
