use core::cmp::max;
use core::cmp::min;
use core::marker::Sized;
use core::result::Result;

use common::string::*;
use common::vector::*;
use common::url::*;

use drivers::keyboard::*;
use drivers::mouse::*;

use graphics::color::*;
use graphics::display::*;
use graphics::point::*;
use graphics::size::*;

use alloc::boxed::*;

pub trait SessionDevice {
    fn on_irq(&mut self, session: &Session, irq: u8);
}

pub trait SessionItem {
    fn new(file: String) -> Self where Self:Sized;
    fn draw(&mut self, session: &Session, &mut SessionUpdates) -> bool;
    fn on_key(&mut self, session: &Session, &mut SessionUpdates, key_event: KeyEvent);
    fn on_mouse(&mut self, session: &Session, &mut SessionUpdates, mouse_event: MouseEvent, alloc_catch: bool) -> bool;
}

pub trait SessionScheme {
    fn scheme(&self) -> String;
    fn on_url(&mut self, session: &Session, url: &URL) -> String;
}

pub const REDRAW_NONE: usize = 0;
pub const REDRAW_CURSOR: usize = 1;
pub const REDRAW_ALL: usize = 2;

pub struct Session {
    pub display: Display,
    pub mouse_point: Point,
    pub devices: Vector<Box<SessionDevice>>,
    pub items: Vector<Box<SessionItem>>,
    pub schemes: Vector<Box<SessionScheme>>,
    pub redraw: usize
}

pub struct SessionUpdates {
    pub new_items: Vector<Box<SessionItem>>,
    pub redraw: usize
}

impl Session {
    pub fn new() -> Session {
        Session {
            display: Display::new(),
            mouse_point: Point::new(0, 0),
            devices: Vector::new(),
            items: Vector::new(),
            schemes: Vector::new(),
            redraw: REDRAW_ALL
        }
    }

    pub fn on_irq(&self, irq: u8){
        for i in 0..self.devices.len(){
            match self.devices.get(i){
                Result::Ok(device) => {
                    device.on_irq(self, irq);
                },
                Result::Err(_) => ()
            }
        }
    }

    pub fn on_url(&self, url: &URL) -> String {
        let mut ret = String::new();

        for i in 0..self.schemes.len(){
            match self.schemes.get(i){
                Result::Ok(scheme) => {
                    if scheme.scheme() == url.scheme {
                        ret = ret + scheme.on_url(self, url);
                    }
                },
                Result::Err(_) => ()
            }
        }

        return ret;
    }

    pub unsafe fn on_key(&mut self, key_event: KeyEvent){
        let mut updates = self.new_updates();

        match self.items.get(0){
            Result::Ok(item) => {
                item.on_key(self, &mut updates, key_event);
                updates.redraw = REDRAW_ALL;
            },
            Result::Err(_) => ()
        }

        self.apply_updates(updates);
    }

    pub unsafe fn on_mouse(&mut self, mouse_event: MouseEvent){
        self.mouse_point.x = max(0, min(self.display.width as isize - 1, self.mouse_point.x + mouse_event.x));
        self.mouse_point.y = max(0, min(self.display.height as isize - 1, self.mouse_point.y + mouse_event.y));

        let mut updates = self.new_updates();
        updates.redraw = REDRAW_CURSOR;

        let mut catcher = 0;
        let mut allow_catch = true;
        for i in 0..self.items.len() {
            match self.items.get(i){
                Result::Ok(item) => if item.on_mouse(self, &mut updates, mouse_event, allow_catch) {
                    allow_catch = false;
                    catcher = i;
                    updates.redraw = REDRAW_ALL;
                },
                Result::Err(_) => ()
            }
        }
        if catcher > 0 {
            match self.items.extract(catcher){
                Result::Ok(item) => self.items.insert(0, item),
                Result::Err(_) => ()
            }
        }

        self.apply_updates(updates);
    }

    pub unsafe fn redraw(&mut self){
        if self.redraw > REDRAW_NONE {
            let mut updates = self.new_updates();

            if self.redraw >= REDRAW_ALL {
                self.display.background();

                self.display.rect(Point::new(0, 0), Size::new(self.display.width, 18), Color::new(0, 0, 0));
                self.display.text(Point::new(self.display.width as isize/ 2 - 3*8, 1), &String::from_str("Redox"), Color::new(255, 255, 255));

                let mut erase_i: Vector<usize> = Vector::new();
                for reverse_i in 0..self.items.len() {
                    let i = self.items.len() - 1 - reverse_i;
                    match self.items.get(i) {
                        Result::Ok(item) => if ! item.draw(self, &mut updates) {
                            erase_i.push(i);
                        },
                        Result::Err(_) => ()
                    }
                }

                for i in erase_i.as_slice() {
                    self.items.erase(*i);
                }
            }

            self.display.flip();

            self.display.cursor(self.mouse_point);

            self.redraw = REDRAW_NONE;

            self.apply_updates(updates);
        }
    }

    fn new_updates(&self) -> SessionUpdates {
        SessionUpdates{
            new_items: Vector::new(),
            redraw: REDRAW_NONE
        }
    }

    fn apply_updates(&mut self, mut updates: SessionUpdates){
        while updates.new_items.len() > 0 {
            match updates.new_items.extract(0){
                Result::Ok(item) => {
                    self.items.insert(0, item);
                    updates.redraw = REDRAW_ALL;
                },
                Result::Err(_) => ()
            }
        }
        self.redraw = max(updates.redraw, self.redraw);
    }
}
