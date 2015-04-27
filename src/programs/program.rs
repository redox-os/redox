use drivers::keyboard::*;
use drivers::mouse::*;

use graphics::display::*;
use graphics::point::*;

pub trait Program {    
    unsafe fn draw(&self, display: &Display);
    unsafe fn on_key(&mut self, key_event: KeyEvent);
    unsafe fn on_mouse(&mut self, mouse_point: Point, mouse_event: MouseEvent) -> bool;
}