use core::cmp::max;
use core::cmp::min;
use core::marker::Sized;
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
    fn draw(&mut self, session: &mut Session) -> bool;
    fn on_key(&mut self, session: &mut Session, key_event: KeyEvent);
    fn on_mouse(&mut self, session: &mut Session, mouse_event: MouseEvent, alloc_catch: bool) -> bool;
}

pub const REDRAW_NONE: usize = 0;
pub const REDRAW_CURSOR: usize = 1;
pub const REDRAW_ALL: usize = 2;

pub struct Session {
    pub display: Display,
    pub mouse_point: Point,
    pub items: Vector<Box<SessionItem>>,
    pub new_items: Vector<Box<SessionItem>>,
    pub redraw: usize
}

impl Session {
    pub unsafe fn new() -> Session {
        Session {
            display: Display::new(),
            mouse_point: Point::new(0, 0),
            items: Vector::<Box<SessionItem>>::new(),
            new_items: Vector::<Box<SessionItem>>::new(),
            redraw: REDRAW_ALL
        }
    }

    pub fn copy_items(&self) -> Vector<Box<SessionItem>>{
        let mut ret: Vector<Box<SessionItem>> = Vector::new();
        for item in self.items.as_slice() {
            unsafe {
                ret = ret + Vector::from_ptr(item);
            }
        }
        return ret;
    }

    pub fn sync_new(&mut self){
        while self.new_items.len() > 0 {
            match self.new_items.remove(0){
                Result::Ok(item) => self.items.insert(0, item),
                Result::Err(_) => ()
            }
        }
    }

    pub unsafe fn on_key(&mut self, key_event: KeyEvent){
        let items = self.copy_items();
        for item in items.as_slice() {
            item.on_key(self, key_event);
            self.redraw = REDRAW_ALL;
            break;
        }

        self.sync_new();
    }

    pub unsafe fn on_mouse(&mut self, mouse_event: MouseEvent){
        self.mouse_point.x = max(0, min(self.display.width as isize - 1, self.mouse_point.x + mouse_event.x));
        self.mouse_point.y = max(0, min(self.display.height as isize - 1, self.mouse_point.y + mouse_event.y));

        self.redraw = max(self.redraw, REDRAW_CURSOR);

        let items = self.copy_items();
        let mut catcher = 0;
        let mut allow_catch = true;
        for i in 0..items.len() {
            match items.get(i){
                Result::Ok(item) => if item.on_mouse(self, mouse_event, allow_catch) {
                    allow_catch = false;
                    catcher = i;
                    self.redraw = REDRAW_ALL;
                },
                Result::Err(_) => ()
            }
        }
        if catcher > 0 {
            match self.items.remove(catcher){
                Result::Ok(item) => self.items.insert(0, item),
                Result::Err(_) => ()
            }
        }

        self.sync_new();
    }

    pub unsafe fn redraw(&mut self){
        if self.redraw > REDRAW_NONE {
            if self.redraw >= REDRAW_ALL {
                self.display.background();

                self.display.rect(Point::new(0, 0), Size::new(self.display.width, 18), Color::new(0, 0, 0));

                self.display.text(Point::new(self.display.width as isize/ 2 - 3*8, 1), &String::from_str("Redox"), Color::new(255, 255, 255));

                let items = self.copy_items();
                for reverse_i in 0..items.len() {
                    let i = items.len() - 1 - reverse_i;
                    match items.get(i) {
                        Result::Ok(item) => if ! item.draw(self) {
                            self.items.remove(i);
                        },
                        Result::Err(_) => ()
                    }
                }
            }

            self.display.flip();

            self.display.cursor(self.mouse_point);

            self.redraw = REDRAW_NONE;
        }
    }
}
