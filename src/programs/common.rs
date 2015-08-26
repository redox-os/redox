pub use alloc::arc::*;
pub use alloc::boxed::*;

pub use core::cmp::max;
pub use core::cmp::min;
pub use core::clone::Clone;
pub use core::mem::size_of;
pub use core::option::Option;
pub use core::ptr;

pub use common::debug::*;
pub use common::event::*;
pub use common::queue::*;
pub use common::mutex::*;
pub use common::random::*;
pub use common::resource::*;
pub use common::scheduler::*;
pub use common::string::*;
pub use common::vec::*;

pub use graphics::display::*;
pub use graphics::point::*;

pub unsafe extern "cdecl" fn item_main(item_ptr: usize){
    let mut session_item = ptr::read(item_ptr as *mut Arc<SessionItem>);
    Arc::unsafe_get_mut(&mut session_item).main();
}

#[allow(unused_variables)]
pub trait SessionItem : ::mopa::Any {
    fn main(&mut self){
        d("No main!\n");
    }

    fn load(&mut self, url: &URL){

    }

    fn draw(&self, display: &Display) -> bool {
        return true;
    }

    fn on_key(&mut self, key_event: KeyEvent){

    }

    fn on_mouse(&mut self, mouse_event: MouseEvent, allow_catch: bool) -> bool{
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
