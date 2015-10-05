use core::char;

use common::string::*;

use syscall::call::*;

/// An optional event
pub enum EventOption {
    Mouse(MouseEvent),
    Key(KeyEvent),
    Redraw(RedrawEvent),
    Open(OpenEvent),
    Unknown(Event),
    None,
}

/// An event
#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Event {
    pub code: char,
    pub a: isize,
    pub b: isize,
    pub c: isize,
    pub d: isize,
    pub e: isize,
}

impl Event {
    //// Create a null event
    pub fn new() -> Event {
        Event {
            code: '\0',
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
        }
    }

    /// Convert the event ot an optional event
    // TODO: Consider doing this via a From trait.
    pub fn to_option(self) -> EventOption {
        match self.code {
            'm' => EventOption::Mouse(MouseEvent::from_event(self)),
            'k' => EventOption::Key(KeyEvent::from_event(self)),
            'r' => EventOption::Redraw(RedrawEvent::from_event(self)),
            'o' => EventOption::Open(OpenEvent::from_event(self)),
            '\0' => EventOption::None,
            _ => EventOption::Unknown(self),
        }
    }

    /// Event trigger
    pub fn trigger(&self) {
        unsafe {
            sys_trigger(self);
        }
    }
}

/// A event related to the mouse
#[derive(Copy, Clone)]
pub struct MouseEvent {
    pub x: isize,
    pub y: isize,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
}

impl MouseEvent {
    /// Convert to an `Event`
    pub fn to_event(&self) -> Event {
        Event {
            code: 'm',
            a: self.x,
            b: self.y,
            c: self.left_button as isize,
            d: self.middle_button as isize,
            e: self.right_button as isize,
        }
    }

    /// Convert an `Event` to a `MouseEvent`
    pub fn from_event(event: Event) -> MouseEvent {
        MouseEvent {
            x: event.a,
            y: event.b,
            left_button: event.c > 0,
            middle_button: event.d > 0,
            right_button: event.e > 0,
        }
    }

    /// Mouse event trigger
    #[inline]
    pub fn trigger(&self) {
        self.to_event().trigger();
    }
}

pub const K_ESC: u8 = 0x01;
pub const K_BKSP: u8 = 0x0E;
pub const K_TAP: u8 = 0x0F;
pub const K_CTRL: u8 = 0x1D;
pub const K_ALT: u8 = 0x38;
pub const K_F1: u8 = 0x3B;
pub const K_F2: u8 = 0x3C;
pub const K_F3: u8 = 0x3D;
pub const K_F4: u8 = 0x3E;
pub const K_F5: u8 = 0x3F;
pub const K_F6: u8 = 0x40;
pub const K_F7: u8 = 0x41;
pub const K_F8: u8 = 0x42;
pub const K_F9: u8 = 0x43;
pub const K_F10: u8 = 0x44;
pub const K_HOME: u8 = 0x47;
pub const K_UP: u8 = 0x48;
pub const K_PGUP: u8 = 0x49;
pub const K_LEFT: u8 = 0x4B;
pub const K_RIGHT: u8 = 0x4D;
pub const K_END: u8 = 0x4F;
pub const K_DOWN: u8 = 0x50;
pub const K_PGDN: u8 = 0x51;
pub const K_DEL: u8 = 0x53;
pub const K_F11: u8 = 0x57;
pub const K_F12: u8 = 0x58;

/// A key event (such as a pressed key)
#[derive(Copy, Clone)]
pub struct KeyEvent {
    pub character: char,
    pub scancode: u8,
    pub pressed: bool,
}

impl KeyEvent {
    /// Convert to an `Event`
    pub fn to_event(&self) -> Event {
        Event {
            code: 'k',
            a: self.character as isize,
            b: self.scancode as isize,
            c: self.pressed as isize,
            d: 0,
            e: 0,
        }
    }

    /// Convert from an `Event`
    pub fn from_event(event: Event) -> KeyEvent {
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
            },
        }
    }

    /// Key event trigger
    #[inline]
    pub fn trigger(&self) {
        self.to_event().trigger();
    }
}

pub const REDRAW_NONE: usize = 0;
pub const REDRAW_CURSOR: usize = 1;
pub const REDRAW_ALL: usize = 2;

/// A redraw event
pub struct RedrawEvent {
    pub redraw: usize,
}

impl RedrawEvent {
    /// Convert to an `Event`
    pub fn to_event(&self) -> Event {
        Event {
            code: 'r',
            a: self.redraw as isize,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
        }
    }

    /// Convert from an `Event`
    pub fn from_event(event: Event) -> RedrawEvent {
        RedrawEvent { redraw: event.a as usize }
    }

    /// Redraw trigger
    #[inline]
    pub fn trigger(&self) {
        self.to_event().trigger();
    }
}

/// A "open event" (such as a IO request)
pub struct OpenEvent {
    /// The URL, see wiki.
    pub url_string: String,
}

impl OpenEvent {
    /// Convert to an `Event`
    pub fn to_event(&self) -> Event {
        unsafe {
            Event {
                code: 'o',
                a: self.url_string.to_c_str() as isize,
                b: 0,
                c: 0,
                d: 0,
                e: 0,
            }
        }
    }

    /// Convert from an `Event`
    pub fn from_event(event: Event) -> OpenEvent {
        unsafe {
            let ret = OpenEvent { url_string: String::from_c_str(event.a as *const u8) };
            sys_unalloc(event.a as usize);
            ret
        }
    }

    /// Event trigger
    pub fn trigger(&self) {
        self.to_event().trigger();
    }
}
