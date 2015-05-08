#![feature(asm)]
#![feature(box_syntax)]
#![feature(core)]
#![feature(lang_items)]
#![feature(no_std)]
#![no_std]

extern crate core;

use core::mem::size_of;

use common::debug::*;
use common::memory::*;
use common::string::*;

use drivers::keyboard::*;
use drivers::mouse::*;

use graphics::color::*;
use graphics::point::*;
use graphics::size::*;
use graphics::window::*;

use programs::program::*;

mod common {
    pub mod debug;
    pub mod memory;
    pub mod pio;
    pub mod string;
    pub mod vector;
}

mod drivers {
    pub mod disk;
    pub mod keyboard;
    pub mod mouse;
}

mod filesystems {
    pub mod unfs;
}

mod graphics {
    pub mod bmp;
    pub mod color;
    pub mod display;
    pub mod point;
    pub mod size;
    pub mod window;
}

mod programs {
    pub mod program;
}

static mut window: *mut Window = 0 as *mut Window;

#[no_mangle]
pub unsafe fn entry(){
    d("Entry\n");
    window = alloc(size_of::<Window>()) as *mut Window;
    *window = Window{
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
    };

    (*window).title.d();
    dl();
}

#[no_mangle]
pub unsafe fn draw(session: &mut Session) -> bool{
    let display = &session.display;

    if window as usize > 0 {
        return (*window).draw(display);
    }

    return false;
}

#[no_mangle]
#[allow(unused_variables)]
pub unsafe fn on_key(session: &mut Session, key_event: KeyEvent){
    if window as usize > 0{
        match key_event.scancode {
            0x01 => (*window).closed = true,
            _ => ()
        }
    }
}

#[no_mangle]
pub unsafe fn on_mouse(session: &mut Session, mouse_event: MouseEvent, allow_catch: bool) -> bool{
    if window as usize > 0 {
        return (*window).on_mouse(session.mouse_point, mouse_event, allow_catch);
    }else{
        return false;
    }
}
