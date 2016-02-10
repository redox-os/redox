use super::{Resource, Url};

use alloc::boxed::Box;

use system::error::{Error, Result, ENOENT};

#[allow(unused_variables)]
pub trait KScheme {
    fn on_irq(&mut self, irq: u8) {

    }

    fn on_poll(&mut self) {

    }

    fn scheme(&self) -> &str {
        ""
    }

    fn open(&mut self, path: &Url, flags: usize) -> Result<Box<Resource>> {
        Err(Error::new(ENOENT))
    }

    fn mkdir(&mut self, path: &Url, flags: usize) -> Result<()> {
        Err(Error::new(ENOENT))
    }

    fn unlink(&mut self, path: &Url) -> Result<()> {
        Err(Error::new(ENOENT))
    }
}
