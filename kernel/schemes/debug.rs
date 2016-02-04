use alloc::boxed::Box;

use collections::string::String;

use arch::context::context_switch;

use fs::{KScheme, Resource, Url};

use system::error::Result;

/// A debug resource
pub struct DebugResource {
    pub command: String,
    pub line_toggle: bool,
}

impl Resource for DebugResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box DebugResource {
            command: self.command.clone(),
            line_toggle: self.line_toggle,
        })
    }

    fn path(&self, buf: &mut [u8]) -> Result <usize> {
        let path = b"debug:";

        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }

        Ok(i)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.line_toggle {
            self.line_toggle = false;
            return Ok(0);
        }

        if self.command.is_empty() {
            loop {
                {
                    let mut console = ::env().console.lock();

                    if console.command.is_some() {
                        if let Some(ref command) = console.command {
                            self.command = command.clone();
                        }
                        console.command = None;
                        break;
                    }
                }

                unsafe { context_switch(false) };
            }
        }

        // TODO: Unicode
        let mut i = 0;
        while i < buf.len() && !self.command.is_empty() {
            buf[i] = unsafe { self.command.as_mut_vec().remove(0) };
            i += 1;
        }

        if i > 0 && self.command.is_empty() {
            self.line_toggle = true;
        }

        Ok(i)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        ::env().console.lock().write(buf);
        Ok(buf.len())
    }

    fn sync(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct DebugScheme;

impl DebugScheme {
    pub fn new() -> Box<Self> {
        box DebugScheme
    }
}

impl KScheme for DebugScheme {
    fn scheme(&self) -> &str {
        "debug"
    }

    fn open(&mut self, _: &Url, _: usize) -> Result<Box<Resource>> {
        Ok(box DebugResource {
            command: String::new(),
            line_toggle: false,
        })
    }
}
