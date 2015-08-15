use core::any::Any;
use core::cmp::max;
use core::cmp::min;
use core::marker::Sized;
use core::option::Option;

use alloc::boxed::*;
use alloc::rc::*;

use common::debug::*;
use common::resource::*;
use common::string::*;
use common::vec::*;
use common::url::*;

use drivers::keyboard::*;
use drivers::mouse::*;

use graphics::color::*;
use graphics::display::*;
use graphics::point::*;
use graphics::size::*;

#[allow(unused_variables)]
pub trait SessionModule {
    fn on_irq(&mut self, session: &Session, updates: &mut SessionUpdates, irq: u8){

    }

    fn on_poll(&mut self, session: &Session, updates: &mut SessionUpdates){

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

    fn load(&mut self, session: &Session, file: String){

    }

    fn draw(&mut self, session: &Session, updates: &mut SessionUpdates) -> bool{
        return true;
    }

    fn on_key(&mut self, session: &Session, updates: &mut SessionUpdates, key_event: KeyEvent){

    }

    fn on_mouse(&mut self, session: &Session, updates: &mut SessionUpdates, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return false;
    }

    fn on_response(&mut self, response: String, callback: Box<FnBox(&mut SessionItem, String)>) where Self:Sized{
        callback.call_box((self, response,));
    }
}
mopafy!(SessionItem, core=core, alloc=alloc);

pub struct OpenEvent {
    pub item: Rc<SessionItem>,
    pub filename: String
}

pub const REDRAW_NONE: usize = 0;
pub const REDRAW_CURSOR: usize = 1;
pub const REDRAW_ALL: usize = 2;

pub struct Session {
    pub display: Display,
    pub mouse_point: Point,
    pub items: Vec<Rc<SessionItem>>,
    pub current_item: isize,
    pub modules: Vec<Rc<SessionModule>>,
    pub redraw: usize
}

pub struct SessionUpdates {
    pub events: Vec<Box<Any>>,
    pub redraw: usize
}

impl Session {
    pub fn new() -> Session {
        Session {
            display: Display::new(),
            mouse_point: Point::new(0, 0),
            items: Vec::new(),
            current_item: -1,
            modules: Vec::new(),
            redraw: REDRAW_ALL
        }
    }

    pub fn on_irq(&mut self, irq: u8){
        let mut updates = self.new_updates();

        for module in self.modules.iter() {
            unsafe{
                Rc::unsafe_get_mut(module).on_irq(self, &mut updates, irq);
            }
        }

        self.apply_updates(updates);
    }

    pub fn on_poll(&mut self){
        let mut updates = self.new_updates();

        for module in self.modules.iter() {
            unsafe{
                Rc::unsafe_get_mut(module).on_poll(self, &mut updates);
            }
        }

        self.apply_updates(updates);
    }

    pub fn open(&self, url: &URL) -> Box<Resource>{
        for module in self.modules.iter() {
            if module.scheme() == url.scheme {
                unsafe{
                    return Rc::unsafe_get_mut(module).open(url);
                }
            }
        }
        return box NoneResource;
    }

    pub fn open_async(&self, url: &URL, callback: Box<FnBox(Box<Resource>)>){
        for module in self.modules.iter() {
            if module.scheme() == url.scheme {
                unsafe{
                    Rc::unsafe_get_mut(module).open_async(url, callback);
                }
                break;
            }
        }
    }

    pub fn on_key(&mut self, key_event: KeyEvent){
        let mut updates = self.new_updates();

        self.current_item = 0;
        match self.items.get(self.current_item as usize){
            Option::Some(item) => {
                unsafe {
                    Rc::unsafe_get_mut(item).on_key(self, &mut updates, key_event);
                }
                updates.redraw = REDRAW_ALL;
            },
            Option::None => ()
        }
        self.current_item = -1;

        self.apply_updates(updates);
    }

    pub fn on_mouse(&mut self, mouse_event: MouseEvent){
        self.mouse_point.x = max(0, min(self.display.width as isize - 1, self.mouse_point.x + mouse_event.x));
        self.mouse_point.y = max(0, min(self.display.height as isize - 1, self.mouse_point.y + mouse_event.y));

        let mut updates = self.new_updates();
        updates.redraw = REDRAW_CURSOR;

        let mut catcher = 0;
        let mut allow_catch = true;
        for i in 0..self.items.len() {
            self.current_item = i as isize;
            match self.items.get(self.current_item as usize){
                Option::Some(item) => {
                    unsafe {
                        if Rc::unsafe_get_mut(item).on_mouse(self, &mut updates, mouse_event, allow_catch) {
                            allow_catch = false;
                            catcher = i;
                            updates.redraw = REDRAW_ALL;
                        }
                    }
                },
                Option::None => ()
            }
        }
        self.current_item = -1;

        if catcher > 0 && catcher < self.items.len() {
            match self.items.remove(catcher){
                Option::Some(item) => {
                    self.items.insert(0, item);
                },
                Option::None => ()
            }
        }

        self.apply_updates(updates);
    }

    pub fn redraw(&mut self){
        if self.redraw > REDRAW_NONE {
            let mut updates = self.new_updates();

            if self.redraw >= REDRAW_ALL {
                self.display.background();

                self.display.rect(Point::new(0, 0), Size::new(self.display.width, 18), Color::new(0, 0, 0));
                self.display.text(Point::new(self.display.width as isize/ 2 - 3*8, 1), &String::from_str("Redox"), Color::new(255, 255, 255));

                let mut erase_i: Vec<usize> = Vec::new();
                for reverse_i in 0..self.items.len() {
                    self.current_item = (self.items.len() - 1 - reverse_i) as isize;
                    match self.items.get(self.current_item as usize) {
                        Option::Some(item) => {
                            unsafe {
                                if ! Rc::unsafe_get_mut(item).draw(self, &mut updates) {
                                    erase_i.push(self.current_item as usize);
                                }
                            }
                        },
                        Option::None => ()
                    }
                }
                self.current_item = -1;

                for i in erase_i.iter() {
                    drop(self.items.remove(*i));
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
            events: Vec::new(),
            redraw: REDRAW_NONE
        }
    }

    fn apply_updates(&mut self, mut updates: SessionUpdates){
        while updates.events.len() > 0 {
            match updates.events.remove(0){
                Option::Some(event) => {
                    match event.downcast_ref::<KeyEvent>() {
                        Option::Some(key_event) => {
                            self.on_key(*key_event);
                            continue;
                        },
                        Option::None => ()
                    }

                    match event.downcast_ref::<MouseEvent>() {
                        Option::Some(mouse_event) => {
                            self.on_mouse(*mouse_event);
                            continue;
                        },
                        Option::None => ()
                    }

                    match event.downcast_ref::<OpenEvent>() {
                        Option::Some(open_event) => {
                            self.items.insert(0, open_event.item.clone());
                            self.current_item = 0;
                            unsafe{
                                Rc::unsafe_get_mut(&open_event.item).load(self, open_event.filename.clone());
                            }
                            self.current_item = -1;
                            updates.redraw = REDRAW_ALL;
                            continue;
                        },
                        Option::None => ()
                    }
                },
                Option::None => ()
            }
        }

        self.redraw = max(updates.redraw, self.redraw);
    }
}
