use super::Resource;

use alloc::boxed::Box;

use system::error::{Error, Result, EPERM};

#[allow(unused_variables)]
pub trait KScheme {
    fn on_irq(&mut self, irq: u8) {

    }

    fn scheme(&self) -> &str {
        ""
    }

    fn open(&mut self, path: &str, flags: usize) -> Result<Box<Resource>> {
        Err(Error::new(EPERM))
    }

    fn mkdir(&mut self, path: &str, flags: usize) -> Result<()> {
        Err(Error::new(EPERM))
    }

    fn rmdir(&mut self, path: &str) -> Result<()> {
        Err(Error::new(EPERM))
    }

    fn unlink(&mut self, path: &str) -> Result<()> {
        Err(Error::new(EPERM))
    }
}
