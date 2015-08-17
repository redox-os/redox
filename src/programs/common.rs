pub use alloc::boxed::*;
pub use alloc::rc::*;

pub use core::clone::Clone;
pub use core::option::Option;

pub use common::debug::*;
pub use common::event::*;
pub use common::random::*;
pub use common::resource::*;
pub use common::string::*;
pub use common::vec::*;

pub use graphics::display::*;
pub use graphics::point::*;

#[allow(unused_variables)]
pub trait SessionModule {
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

    fn open_async(&mut self, url: &URL, callback: Box<FnBox(Box<Resource>)>) {
        callback(self.open(url));
    }
}

#[allow(unused_variables)]
pub trait SessionItem : ::mopa::Any {
    fn new() -> Self where Self:Sized;

    fn load(&mut self, url: &URL){

    }

    fn draw(&mut self, display: &Display) -> bool{
        return true;
    }

    fn on_key(&mut self, key_event: KeyEvent){

    }

    fn on_mouse(&mut self, mouse_point: Point, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return false;
    }
}
mopafy!(SessionItem, core=core, alloc=alloc);
