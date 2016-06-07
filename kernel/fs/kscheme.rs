use super::{Resource, Url};

use alloc::boxed::Box;

use system::error::{EPERM, Error, Result};
use system::syscall::Stat;

#[allow(unused_variables)]
pub trait KScheme {
    fn on_irq(&mut self, irq: u8) {}

    fn scheme(&self) -> &str {
        ""
    }

    fn open(&mut self, path: Url, flags: usize) -> Result<Box<Resource>> {
        Err(Error::new(EPERM))
    }

    fn mkdir(&mut self, path: Url, flags: usize) -> Result<()> {
        Err(Error::new(EPERM))
    }

    fn rmdir(&mut self, path: Url) -> Result<()> {
        Err(Error::new(EPERM))
    }

    fn stat(&mut self, path: Url, stat: &mut Stat) -> Result<()> {
        Err(Error::new(EPERM))
    }

    fn unlink(&mut self, path: Url) -> Result<()> {
        Err(Error::new(EPERM))
    }
}
