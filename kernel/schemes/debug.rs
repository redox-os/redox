use alloc::boxed::Box;

use collections::borrow::ToOwned;
use collections::string::String;

use core::cmp;

use fs::{KScheme, Resource, Url};

use system::error::Result;

/// A debug resource
pub struct DebugResource {
    pub path: String,
    pub command: String,
}

impl Resource for DebugResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box DebugResource {
            path: self.path.clone(),
            command: self.command.clone(),
        })
    }

    fn path(&self, buf: &mut [u8]) -> Result <usize> {
        let path = self.path.as_bytes();

        for (b, p) in buf.iter_mut().zip(path.iter()) {
            *b = *p;
        }

        Ok(cmp::min(buf.len(), path.len()))
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.command.is_empty() {
            self.command = unsafe { &mut *::env().console.get() }.commands.receive("DebugResource::read");
        }

        let mut i = 0;
        while i < buf.len() && ! self.command.is_empty() {
            buf[i] = unsafe { self.command.as_mut_vec().remove(0) };
            i += 1;
        }

        Ok(i)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        unsafe { &mut *::env().console.get() }.write(buf);
        Ok(buf.len())
    }

    fn sync(&mut self) -> Result<()> {
        let console = unsafe { &mut *::env().console.get() };
        console.redraw = true;
        console.write(&[]);
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
        let console = unsafe { & *::env().console.get() };
        if let Some(ref display) = console.display {
            Ok(box DebugResource {
                path: format!("debug:{}/{}", display.width/8, display.height/16),
                command: String::new()
            })
        } else {
            Ok(box DebugResource {
                path: "debug:".to_owned(),
                command: String::new()
            })
        }
    }
}
