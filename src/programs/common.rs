pub use alloc::boxed::*;
pub use alloc::rc::*;

pub use core::any::Any;

pub use common::resource::*;
pub use common::string::*;
pub use common::vec::*;

pub use drivers::mouse::MouseEvent;
pub use drivers::keyboard::KeyEvent;

pub use graphics::display::*;
pub use graphics::point::*;

pub struct OpenEvent {
    pub item: Rc<SessionItem>,
    pub url: URL
}

pub const REDRAW_NONE: usize = 0;
pub const REDRAW_CURSOR: usize = 1;
pub const REDRAW_ALL: usize = 2;

pub struct RedrawEvent {
    pub redraw: usize
}

#[allow(unused_variables)]
pub trait SessionModule {
    fn on_irq(&mut self, events: &mut Vec<Box<Any>>, irq: u8){

    }

    fn on_poll(&mut self, events: &mut Vec<Box<Any>>){

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

    fn draw(&mut self, display: &Display, events: &mut Vec<Box<Any>>) -> bool{
        return true;
    }

    fn on_key(&mut self, events: &mut Vec<Box<Any>>, key_event: KeyEvent){

    }

    fn on_mouse(&mut self, events: &mut Vec<Box<Any>>, mouse_point: Point, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return false;
    }
}
mopafy!(SessionItem, core=core, alloc=alloc);
