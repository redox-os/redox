use core::cmp::max;
use core::cmp::min;
use core::marker::Sized;
use core::ptr;
use core::result::Result;

use common::string::*;
use common::vector::*;

use drivers::keyboard::*;
use drivers::mouse::*;

use graphics::color::*;
use graphics::display::*;
use graphics::point::*;
use graphics::size::*;

use alloc::boxed::*;

pub trait SessionItem {
    fn new(file: String) -> Self where Self:Sized;
    unsafe fn draw(&mut self, session: &mut Session) -> bool{
        false
    }
    unsafe fn on_key(&mut self, session: &mut Session, key_event: KeyEvent){

    }
    unsafe fn on_mouse(&mut self, session: &mut Session, mouse_event: MouseEvent, alloc_catch: bool) -> bool{
        false
    }
}

pub const REDRAW_NONE: usize = 0;
pub const REDRAW_CURSOR: usize = 1;
pub const REDRAW_ALL: usize = 2;

pub struct Session {
    pub display: Display,
    pub mouse_point: Point,
    pub items: Vector<Box<SessionItem>>,
    pub redraw: usize
}

impl Session {
    pub unsafe fn new() -> Session {
        Session {
            display: Display::new(),
            mouse_point: Point::new(0, 0),
            items: Vector::<Box<SessionItem>>::new(),
            redraw: REDRAW_ALL
        }
    }

    pub fn copy_items(&self) -> Vector<Box<SessionItem>>{
        let mut ret: Vector<Box<SessionItem>> = Vector::<Box<SessionItem>>::new();
        for item in self.items.as_slice() {
            unsafe {
                ret = ret + Vector::<Box<SessionItem>>::from_ptr(item);
            }
        }
        return ret;
    }

    pub fn add_item(&mut self, item: Box<SessionItem>){
        let mut new_items = self.copy_items();
        new_items = Vector::<Box<SessionItem>>::from_value(item) + new_items;
        self.items = new_items;
        self.redraw = REDRAW_ALL;
    }

    pub unsafe fn on_key(&mut self, key_event: KeyEvent){
        let items = self.copy_items();
        for item in items.as_slice() {
            item.on_key(self, key_event);
            self.redraw = REDRAW_ALL;
            break;
        }
    }

    pub unsafe fn on_mouse(&mut self, mouse_event: MouseEvent){
        self.mouse_point.x = max(0, min(self.display.width as isize - 1, self.mouse_point.x + mouse_event.x));
        self.mouse_point.y = max(0, min(self.display.height as isize - 1, self.mouse_point.y + mouse_event.y));

        self.redraw = max(self.redraw, REDRAW_CURSOR);

        let items = self.copy_items();
        let mut new_items = Vector::<Box<SessionItem>>::new();
        let mut allow_catch = true;
        for item in items.as_slice() {
            if item.on_mouse(self, mouse_event, allow_catch) {
                new_items = Vector::<Box<SessionItem>>::from_ptr(item) + new_items;
                allow_catch = false;
                self.redraw = REDRAW_ALL;
            }else{
                new_items = new_items + Vector::<Box<SessionItem>>::from_ptr(item);
            }
        }
        self.items = new_items;
    }

    pub unsafe fn redraw(&mut self){
        if self.redraw > REDRAW_NONE {
            if self.redraw >= REDRAW_ALL {
                self.display.background();

                self.display.rect(Point::new(0, 0), Size::new(self.display.width, 18), Color::new(0, 0, 0));

                self.display.text(Point::new(self.display.width as isize/ 2 - 3*8, 1), &String::from_str("Redox"), Color::new(255, 255, 255));

                let items = self.copy_items();
                let mut new_items = Vector::<Box<SessionItem>>::new();
                for i in 0..items.len() {
                    match items.get(items.len() - 1 - i) {
                        Result::Ok(item) => if item.draw(self){
                            new_items = Vector::<Box<SessionItem>>::from_ptr(item) + new_items;
                        }else{
                            //Destroy item !!!!SHOULD DO THIS BETTER!!!!
                            ptr::read(item);
                        },
                        Result::Err(_) => ()
                    }
                }
                self.items = new_items;
            }

            self.display.flip();

            self.display.cursor(self.mouse_point);

            self.redraw = REDRAW_NONE;
        }
    }
}
