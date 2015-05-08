#![feature(asm)]
#![feature(box_syntax)]
#![feature(core)]
#![feature(lang_items)]
#![feature(no_std)]
#![no_std]

extern crate core;

use core::mem::size_of;

use common::memory::*;
use common::string::*;

use drivers::keyboard::*;
use drivers::mouse::*;

use graphics::color::*;
use graphics::point::*;
use graphics::size::*;
use graphics::window::*;

use programs::session::*;

#[path="../../src/common"]
mod common {
    pub mod debug;
    pub mod memory;
    pub mod pio;
    pub mod string;
    pub mod vector;
}

#[path="../../src/drivers"]
mod drivers {
    pub mod disk;
    pub mod keyboard;
    pub mod mouse;
}

#[path="../../src/filesystems"]
mod filesystems {
    pub mod unfs;
}

#[path="../../src/graphics"]
mod graphics {
    pub mod bmp;
    pub mod color;
    pub mod display;
    pub mod point;
    pub mod size;
    pub mod window;
}

#[path="../../src/programs"]
mod programs {
    pub mod session;
}

pub struct Application {
    window: Window,
    character: char
}

impl Application {
    pub fn new() -> Application {
        Application {
            window: Window{
                point: Point::new(420, 300),
                size: Size::new(576, 400),
                title: String::from_str("Test Application"),
                title_color: Color::new(0, 0, 0),
                border_color: Color::new(196, 196, 255),
                content_color: Color::alpha(128, 128, 196, 196),
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
            character: ' '
        }
    }
}

impl SessionItem for Application {
    unsafe fn draw(&mut self, session: &mut Session) -> bool{
        let display = &session.display;
        if self.window.draw(display) {
            display.char(self.window.point, self.character, Color::new(255, 255, 255));
            return true;
        }else{
            return false;
        }
    }

    #[allow(unused_variables)]
    unsafe fn on_key(&mut self, session: &mut Session, key_event: KeyEvent){
        if key_event.pressed {
            match key_event.scancode {
                0x01 => self.window.closed = true,
                _ => ()
            }

            match key_event.character {
                '\x00' => (),
                '\x1B' => (),
                _ => {
                    self.character = key_event.character
                }
            }
        }
    }

    unsafe fn on_mouse(&mut self, session: &mut Session, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return self.window.on_mouse(session.mouse_point, mouse_event, allow_catch);
    }
}

//Class wrappers

static mut application: *mut Application = 0 as *mut Application;

#[no_mangle]
pub unsafe fn entry(){
    application = alloc(size_of::<Application>()) as *mut Application;
    *application = Application::new();
}

#[no_mangle]
pub unsafe fn draw(session: &mut Session) -> bool{
    if application as usize > 0 {
        return (*application).draw(session);
    }else{
        return false;
    }
}

#[no_mangle]
pub unsafe fn on_key(session: &mut Session, key_event: KeyEvent){
    if application as usize > 0{
        (*application).on_key(session, key_event);
    }
}

#[no_mangle]
pub unsafe fn on_mouse(session: &mut Session, mouse_event: MouseEvent, allow_catch: bool) -> bool{
    if application as usize > 0 {
        return (*application).on_mouse(session, mouse_event, allow_catch);
    }else{
        return false;
    }
}
