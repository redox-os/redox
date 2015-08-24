use core::char;

use common::memory::*;
use common::string::*;

#[derive(Copy, Clone)]
pub struct Event {
    pub code: char,
    pub a: isize,
    pub b: isize,
    pub c: isize,
    pub d: isize,
    pub e: isize
}

impl Event {
    pub fn trigger(&self){
        unsafe{
            let event_ptr: *const Event = self;
            asm!("int 0x80"
                :
                : "{eax}"(2), "{ebx}"(event_ptr as u32)
                :
                : "intel");
        }
    }
}

#[derive(Copy, Clone)]
pub struct MouseEvent {
    pub x: isize,
    pub y: isize,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
    pub valid: bool
}

impl MouseEvent {
    pub fn to_event(&self) -> Event {
        Event {
            code: 'm',
            a: self.x,
            b: self.y,
            c: self.left_button as isize,
            d: self.middle_button as isize,
            e: self.right_button as isize
        }
    }

    pub fn from_event(event: &mut Event) -> MouseEvent {
        MouseEvent {
            x: event.a,
            y: event.b,
            left_button: event.c > 0,
            middle_button: event.d > 0,
            right_button: event.e > 0,
            valid: true
        }
    }

    pub fn trigger(&self){
        self.to_event().trigger();
    }
}

#[derive(Copy, Clone)]
pub struct KeyEvent {
    pub character: char,
    pub scancode: u8,
    pub pressed: bool
}

impl KeyEvent {
    pub fn to_event(&self) -> Event {
        Event {
            code: 'k',
            a: self.character as isize,
            b: self.scancode as isize,
            c: self.pressed as isize,
            d: 0,
            e: 0
        }
    }

    pub fn from_event(event: &mut Event) -> KeyEvent {
        match char::from_u32(event.a as u32) {
            Option::Some(character) => KeyEvent {
                character: character,
                scancode: event.b as u8,
                pressed: event.c > 0,
            },
            Option::None => KeyEvent {
                character: '\0',
                scancode: event.b as u8,
                pressed: event.c > 0,
            }
        }
    }

    pub fn trigger(&self){
        self.to_event().trigger();
    }
}

pub const REDRAW_NONE: usize = 0;
pub const REDRAW_CURSOR: usize = 1;
pub const REDRAW_ALL: usize = 2;

pub struct RedrawEvent {
    pub redraw: usize
}

impl RedrawEvent {
    pub fn to_event(&self) -> Event {
        Event {
            code: 'r',
            a: self.redraw as isize,
            b: 0,
            c: 0,
            d: 0,
            e: 0
        }
    }

    pub fn from_event(event: &mut Event) -> RedrawEvent {
        RedrawEvent {
            redraw: event.a as usize
        }
    }

    pub fn trigger(&self){
        self.to_event().trigger();
    }
}

pub struct OpenEvent {
    pub url_string: String
}

impl OpenEvent {
    pub fn to_event(&self) -> Event {
        unsafe{
            Event {
                code: 'o',
                a: self.url_string.to_c_str() as isize,
                b: 0,
                c: 0,
                d: 0,
                e: 0
            }
        }
    }

    pub fn from_event(event: &mut Event) -> OpenEvent {
        unsafe{
            let ret = OpenEvent {
                url_string: String::from_c_str(event.a as *const u8)
            };
            unalloc(event.a as usize);
            event.a = 0;
            return ret;
        }
    }

    pub fn trigger(&self){
        self.to_event().trigger();
    }
}
