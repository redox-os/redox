#![feature(asm)]
#![feature(box_syntax)]
#![feature(coerce_unsized)]
#![feature(core_simd)]
#![feature(core_slice_ext)]
#![feature(core_str_ext)]
#![feature(fundamental)]
#![feature(lang_items)]
#![feature(no_std)]
#![feature(raw)]
#![feature(unique)]
#![feature(unsize)]
#![no_std]

use core::clone::Clone;
use core::mem::size_of;
use core::result::Result;

use common::memory::*;
use common::string::*;
use common::vector::*;
use common::url::*;

use drivers::keyboard::*;
use drivers::mouse::*;

use graphics::color::*;
use graphics::point::*;
use graphics::size::*;
use graphics::window::*;

use programs::session::*;

/* TEST { */
use core::any::Any;
use core::ops::Fn;
use core::option::Option;

use alloc::boxed::*;

use common::debug::*;
/* } TEST */

#[path="../src/alloc"]
mod alloc {
    pub mod boxed;
}

#[path="../src/common"]
mod common {
    pub mod debug;
    pub mod memory;
    pub mod pci;
    pub mod pio;
    pub mod string;
    pub mod vector;
    pub mod url;
}

#[path="../src/drivers"]
mod drivers {
    pub mod disk;
    pub mod keyboard;
    pub mod mouse;
}

#[path="../src/filesystems"]
mod filesystems {
    pub mod unfs;
}

#[path="../src/graphics"]
mod graphics {
    pub mod bmp;
    pub mod color;
    pub mod display;
    pub mod point;
    pub mod size;
    pub mod window;
}

#[path="../src/programs"]
mod programs {
    pub mod session;
}

/* TEST { */
struct EventA {
    x: isize,
    y: isize
}

struct EventB {
    txt: String
}

struct EventListener {
    fn_ptr: Box<Fn(&Box<Any>)>
}

impl EventListener {
    pub fn call(&self, event: &Box<Any>){
        (*self.fn_ptr)(event);
    }
}

fn test(){
    let mut events: Vector<Box<Any>> = Vector::new();

    events.push(box EventB {
        txt: "first test".to_string()
    });
    events.push(box EventA {
        x: 2,
        y: 3
    });
    events.push(box EventB {
        txt: "second test".to_string()
    });

    let mut listeners: Vector<Box<EventListener>> = Vector::new();

    listeners.push(box EventListener {
        fn_ptr: box |event: &Box<Any>| {
            match event.downcast_ref::<EventA>() {
                Option::Some(a) => {
                    d("Event A ");
                    dd(a.x as usize);
                    d(", ");
                    dd(a.y as usize);
                    dl();
                },
                Option::None => ()
            }
        }
    });

    listeners.push(box EventListener {
        fn_ptr: box |event: &Box<Any>| {
            match event.downcast_ref::<EventB>() {
                Option::Some(b) => {
                    d("Event B ");
                    b.txt.d();
                    dl();
                },
                Option::None => ()
            }
        }
    });

    for event in events.iter() {
        for listener in listeners.iter() {
            listener.call(event);
        }
    }
}
/* } TEST */

pub struct Application {
    window: Window,
    output: String,
    command: String,
    offset: usize,
    scroll: Point,
    wrap: bool
}

impl Application {
    fn append(&mut self, line: String) {
        self.output = self.output.clone() + line + "\n";
    }

    #[allow(unused_variables)]
    fn on_command(&mut self, session: &Session){
        let mut args: Vector<String> = Vector::<String>::new();
        for arg in self.command.split(" ".to_string()) {
            if arg.len() > 0 {
                args.push(arg);
            }
        }
        match args.get(0) {
            Result::Ok(cmd) => {
                if *cmd == "echo".to_string() {
                    let mut echo = String::new();
                    for i in 1..args.len() {
                        match args.get(i) {
                            Result::Ok(arg) => {
                                if echo.len() == 0 {
                                    echo = arg.clone();
                                }else{
                                    echo = echo + " " + arg.clone();
                                }
                            },
                            Result::Err(_) => ()
                        }
                    }
                    self.append(echo);
                }else if *cmd == "exit".to_string() {
                    self.window.closed = true;
                }else if *cmd == "test".to_string() {
                    test();
                }else if *cmd == "url".to_string() {
                    match args.get(1) {
                        Result::Ok(url_string) => {
                            let url = URL::from_string(url_string.clone());
                            self.append(url.to_string());
                            self.append(session.on_url(&url));
                        },
                        Result::Err(_) => {
                            for module in session.modules.iter() {
                                let scheme = module.scheme();
                                if scheme.len() > 0 {
                                    self.append(scheme);
                                }
                            }
                        }
                    }
                }else{
                    self.append("Commands:  echo  exit  url".to_string());
                }
            },
            Result::Err(_) => ()
        }
    }
}

impl SessionItem for Application {
    #[allow(unused_variables)]
    fn new(file: String) -> Application {
        Application {
            window: Window{
                point: Point::new(220, 100),
                size: Size::new(576, 400),
                title: String::from_str("Terminal"),
                title_color: Color::new(0, 0, 0),
                border_color: Color::new(192, 192, 255),
                content_color: Color::alpha(128, 128, 160, 192),
                shaded: false,
                closed: false,
                dragging: false,
                last_mouse_point: Point::new(0, 0),
                last_mouse_event: MouseEvent {
                    x: 0,
                    y: 0,
                    left_button: false,
                    right_button: false,
                    middle_button: false,
                    valid: false
                }
            },
            output: String::new(),
            command: String::new(),
            offset: 0,
            scroll: Point::new(0, 0),
            wrap: true
        }
    }

    fn draw(&mut self, session: &Session, updates: &mut SessionUpdates) -> bool{
        let display = &session.display;
        if self.window.draw(display) {
            let scroll = self.scroll;

            let mut col = -scroll.x;
            let cols = self.window.size.width as isize / 8;
            let mut row = -scroll.y;
            let rows = self.window.size.height as isize / 16;

            for c in self.output.chars(){
                if self.wrap && col >= cols {
                    col = -scroll.x;
                    row += 1;
                }

                if c == '\n' {
                    col = -scroll.x;
                    row += 1;
                }else if c == '\t' {
                    col += 8 - col % 8;
                }else{
                    if col >= 0 && col < cols && row >= 0 && row < rows{
                        let point = Point::new(self.window.point.x + 8 * col, self.window.point.y + 16 * row);
                        display.char(point, c, Color::new(224, 224, 224));
                    }
                    col += 1;
                }
            }

            if col > -scroll.x {
                col = -scroll.x;
                row += 1;
            }

            if col >= 0 && col < cols && row >= 0 && row < rows{
                let point = Point::new(self.window.point.x + 8 * col, self.window.point.y + 16 * row);
                display.char(point, '#', Color::new(255, 255, 255));
                col += 2;
            }

            let mut i = 0;
            for c in self.command.chars(){
                if self.wrap && col >= cols {
                    col = -scroll.x;
                    row += 1;
                }

                if self.offset == i && col >= 0 && col < cols && row >= 0 && row < rows{
                    let point = Point::new(self.window.point.x + 8 * col, self.window.point.y + 16 * row);
                    display.char(point, '_', Color::new(255, 255, 255));
                }

                if c == '\n' {
                    col = -scroll.x;
                    row += 1;
                }else if c == '\t' {
                    col += 8 - col % 8;
                }else{
                    if col >= 0 && col < cols && row >= 0 && row < rows{
                        let point = Point::new(self.window.point.x + 8 * col, self.window.point.y + 16 * row);
                        display.char(point, c, Color::new(255, 255, 255));
                    }
                    col += 1;
                }

                i += 1;
            }

            if self.wrap && col >= cols {
                col = -scroll.x;
                row += 1;
            }

            if row >= rows {
                self.scroll.y += row - rows + 1;
                updates.redraw = REDRAW_ALL;
            }

            if self.offset == i && col >= 0 && col < cols && row >= 0 && row < rows{
                let point = Point::new(self.window.point.x + 8 * col, self.window.point.y + 16 * row);
                display.char(point, '_', Color::new(255, 255, 255));
            }

            return true;
        }else{
            return false;
        }
    }

    #[allow(unused_variables)]
    fn on_key(&mut self, session: &Session, updates: &mut SessionUpdates, key_event: KeyEvent){
        if key_event.pressed {
            match key_event.scancode {
                0x01 => self.window.closed = true,
                0x47 => self.offset = 0,
                0x4B => if self.offset > 0 {
                    self.offset -= 1;
                },
                0x4D => if self.offset < self.command.len() {
                    self.offset += 1;
                },
                0x4F => self.offset = self.command.len(),
                _ => ()
            }

            match key_event.character {
                '\x00' => (),
                '\x08' => {
                    if self.offset > 0 {
                        self.command = self.command.substr(0, self.offset - 1) + self.command.substr(self.offset, self.command.len() - self.offset);
                        self.offset -= 1;
                    }
                }
                '\x1B' => self.command = String::new(),
                '\n' => {
                    if self.command.len() > 0 {
                        self.output = self.output.clone() + "# ".to_string() + self.command.clone() + "\n";
                        self.on_command(session);
                        self.command = String::new();
                        self.offset = 0;
                    }
                },
                _ => {
                    self.command = self.command.substr(0, self.offset) + key_event.character + self.command.substr(self.offset, self.command.len() - self.offset);
                    self.offset += 1;
                }
            }
        }
    }

    #[allow(unused_variables)]
    fn on_mouse(&mut self, session: &Session, updates: &mut SessionUpdates, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return self.window.on_mouse(session.mouse_point, mouse_event, allow_catch);
    }
}

//Class wrappers

static mut application: *mut Application = 0 as *mut Application;

#[no_mangle]
pub unsafe fn entry(){
    application = alloc(size_of::<Application>()) as *mut Application;
    *application = Application::new("".to_string());
}

#[no_mangle]
pub unsafe fn draw(session: &Session, updates: &mut SessionUpdates) -> bool{
    if application as usize > 0 {
        return (*application).draw(session, updates);
    }else{
        return false;
    }
}

#[no_mangle]
pub unsafe fn on_key(session: &Session, updates: &mut SessionUpdates, key_event: KeyEvent){
    if application as usize > 0{
        (*application).on_key(session, updates, key_event);
    }
}

#[no_mangle]
pub unsafe fn on_mouse(session: &Session, updates: &mut SessionUpdates, mouse_event: MouseEvent, allow_catch: bool) -> bool{
    if application as usize > 0 {
        return (*application).on_mouse(session, updates, mouse_event, allow_catch);
    }else{
        return false;
    }
}
