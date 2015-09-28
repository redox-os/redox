pub use alloc::boxed::*;

pub use core::cmp::max;
pub use core::cmp::min;
pub use core::clone::Clone;
pub use core::mem::size_of;
pub use core::mem::size_of_val;
pub use core::option::Option;
pub use core::ptr;
pub use core::sync::atomic::*;

pub use common::debug::*;
pub use common::event::*;
pub use common::queue::*;
pub use common::random::*;
pub use common::resource::*;
pub use common::string::*;
pub use common::time::*;
pub use common::vec::*;

pub use graphics::color::*;
pub use graphics::display::*;
pub use graphics::point::*;
pub use graphics::size::*;
pub use graphics::window::*;

pub use syscall::call::*;

#[allow(unused_variables)]
pub trait SessionItem{
    fn on_irq(&mut self, irq: u8){

    }

    fn on_poll(&mut self){

    }

    fn scheme(&self) -> String{
        return String::new();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        return box NoneResource;
    }
}
