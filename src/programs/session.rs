use core::result::Result;

use common::memory::*;
use common::string::*;
use common::vector::*;

use drivers::keyboard::*;
use drivers::mouse::*;

use graphics::color::*;
use graphics::display::*;
use graphics::point::*;
use graphics::size::*;

pub trait SessionItem {
    unsafe fn draw(&mut self, session: &mut Session) -> bool;
    unsafe fn on_key(&mut self, session: &mut Session, key_event: KeyEvent);
    unsafe fn on_mouse(&mut self, session: &mut Session, mouse_event: MouseEvent, alloc_catch: bool) -> bool;
}

pub struct Session {
    pub display: Display,
    pub mouse_point: Point,
    pub items: Vector<Box<SessionItem>>,
    pub redraw: bool
}

impl Session {
    pub unsafe fn new() -> Session {
        Session {
            display: Display::new(),
            mouse_point: Point::new(0, 0),
            items: Vector::<Box<SessionItem>>::new(),
            redraw: true
        }
    }

    // TODO: Find out how to remove
    pub fn copy_items(&self) -> Vector<Box<SessionItem>>{
        let mut ret: Vector<Box<SessionItem>> = Vector::<Box<SessionItem>>::new();
        for item in self.items.as_slice() {
            ret = ret + Vector::<Box<SessionItem>>::from_ptr(item);
        }
        return ret;
    }

    pub fn add_item(&mut self, item: Box<SessionItem>){
        let mut new_items = self.copy_items();
        new_items = Vector::<Box<SessionItem>>::from_value(item) + new_items;
        self.items = new_items;
        self.redraw = true;
    }

    pub unsafe fn on_key(&mut self, key_event: KeyEvent){
        let items = self.copy_items();
        for item in items.as_slice() {
            (*item).on_key(self, key_event);
            self.redraw = true;
            break;
        }
    }

    pub unsafe fn on_mouse(&mut self, mouse_event: MouseEvent){
        self.mouse_point.x += mouse_event.x;
        if self.mouse_point.x < 0 {
            self.mouse_point.x = 0;
        }
        if self.mouse_point.x >= self.display.width as isize {
            self.mouse_point.x = self.display.width as isize - 1;
        }

        self.mouse_point.y += mouse_event.y;
        if self.mouse_point.y < 0 {
            self.mouse_point.y = 0;
        }
        if self.mouse_point.y >= self.display.height as isize {
            self.mouse_point.y = self.display.height as isize - 1;
        }

        let items = self.copy_items();
        let mut new_items = Vector::<Box<SessionItem>>::new();
        let mut allow_catch = true;
        for item in items.as_slice() {
            if (*item).on_mouse(self, mouse_event, allow_catch) {
                new_items = Vector::<Box<SessionItem>>::from_ptr(item) + new_items;
                allow_catch = false;
            }else{
                new_items = new_items + Vector::<Box<SessionItem>>::from_ptr(item);
            }
        }
        self.items = new_items;

        self.redraw = true;
    }

    pub unsafe fn redraw(&mut self){
        if self.redraw {
            self.redraw = false;

            self.display.background();

            self.display.rect(Point::new(0, 0), Size::new(self.display.width, 18), Color::new(0, 0, 0));

            self.display.text(Point::new(self.display.width as isize/ 2 - 3*8, 1), &String::from_str("Redox"), Color::new(255, 255, 255));

            let items = self.copy_items();
            let mut new_items = Vector::<Box<SessionItem>>::new();
            for i in 0..items.len() {
                match items.get(items.len() - 1 - i) {
                    Result::Ok(item) => if (*item).draw(self){
                        new_items = Vector::<Box<SessionItem>>::from_ptr(item) + new_items;
                    },
                    Result::Err(_) => ()
                }
            }
            self.items = new_items;

            self.display.cursor(self.mouse_point);

            self.display.flip();
        }
    }
}
