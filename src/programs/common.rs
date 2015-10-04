pub use core::clone::Clone;
pub use core::option::Option;
pub use core::sync::atomic::*;

use alloc::boxed::*;

use common::resource::{NoneResource, Resource, URL};
use common::string::String;

#[allow(unused_variables)]
pub trait SessionItem {
    fn on_irq(&mut self, irq: u8) {

    }

    fn on_poll(&mut self) {

    }

    fn scheme(&self) -> String {
        String::new()
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        box NoneResource
    }
}
