pub use alloc::boxed::*;

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

pub fn sched_yield(){
    unsafe {
        asm!("int 0x80"
            : : "{eax}"(3) : : "intel");
    }
}

#[allow(unused_variables)]
pub trait SessionItem : ::mopa::Any {
    fn load(&mut self, url: &URL){

    }

    fn draw(&self, display: &Display) -> bool {
        return true;
    }

    fn on_key(&mut self, key_event: KeyEvent){

    }

    fn on_mouse(&mut self, mouse_point: Point, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return false;
    }

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
mopafy!(SessionItem, core=core, alloc=alloc);
