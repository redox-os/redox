use alloc::boxed::Box;

use collections::string::String;

use core::cmp;

use fs::{KScheme, Resource, Url};

use system::error::Result;

/// A debug resource
pub struct DebugResource {
    pub command: String,
}

impl Resource for DebugResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box DebugResource {
            command: self.command.clone(),
        })
    }

    fn path(&self, buf: &mut [u8]) -> Result <usize> {
        let path = b"debug:";

        for (b, p) in buf.iter_mut().zip(path.iter()) {
            *b = *p;
        }

        Ok(cmp::min(buf.len(), path.len()))
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.command.is_empty() {
            self.command = ::env().console.lock().commands.receive();
        }

        let mut i = 0;
        while i < buf.len() && ! self.command.is_empty() {
            buf[i] = unsafe { self.command.as_mut_vec().remove(0) };
            i += 1;
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

    fn open(&mut self, _: Url, _: usize) -> Result<Box<Resource>> {
        Ok(box DebugResource {
            command: String::new()
        })
    }
}
