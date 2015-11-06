use core::{char, cmp};
use common::prompt::*;

use scheduler;

/// An optional event
pub enum EventOption {
    /// A mouse event
    Mouse(MouseEvent),
    /// A key event
    Key(KeyEvent),
    /// A quit request event
    Quit(QuitEvent),
    /// A unknown event
    Unknown(Event),
    /// No event
    None,
}

/// An event
// TODO: Make this a scheme
#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Event {
    pub code: char,
    pub a: isize,
    pub b: isize,
    pub c: isize,
}

impl Event {
    /// Create a null event
    pub fn new() -> Event {
        Event {
            code: '\0',
            a: 0,
            b: 0,
            c: 0,
        }
    }

    /// Convert the event ot an optional event
    // TODO: Consider doing this via a From trait.
    pub fn to_option(self) -> EventOption {
        match self.code {
            'm' => EventOption::Mouse(MouseEvent::from_event(self)),
            'k' => EventOption::Key(KeyEvent::from_event(self)),
            'q' => EventOption::Quit(QuitEvent::from_event(self)),
            '\0' => EventOption::None,
            _ => EventOption::Unknown(self),
        }
    }

    /// Event trigger
    pub fn trigger(&self) {
        let mut event = *self;

        unsafe {
            let reenable = scheduler::start_no_ints();

//            if event.code == 'm' {
//                event.a = cmp::max(0,
//                                   cmp::min((*::session_ptr).display.width as isize - 1,
//                                            (*::session_ptr).mouse_point.x + event.a));
//                event.b = cmp::max(0,
//                                   cmp::min((*::session_ptr).display.height as isize - 1,
//                                            (*::session_ptr).mouse_point.y + event.b));
//                (*::session_ptr).mouse_point.x = event.a;
//                (*::session_ptr).mouse_point.y = event.b;
//                (*::session_ptr).redraw = true;
//            }
//

            (*::events_ptr).push(event);

            scheduler::end_no_ints(reenable);
        }
    }
}

/// A event related to the mouse
#[derive(Copy, Clone)]
pub struct MouseEvent {
    /// The x coordinate
    pub x: isize,
    /// The y coordinate
    pub y: isize,
    /// Is the left button pressed?
    pub left_button: bool,
    /// Is the midle button pressed?
    pub middle_button: bool,
    /// Is the right button pressed?
    pub right_button: bool,
}

impl MouseEvent {
    /// Convert to an `Event`
    pub fn to_event(&self) -> Event {
        Event {
            code: 'm',
            a: self.x,
            b: self.y,
            c: (self.left_button as isize) | (self.middle_button as isize) << 1 | (self.right_button as isize) << 2,
        }
    }

    /// Convert an `Event` to a `MouseEvent`
    pub fn from_event(event: Event) -> MouseEvent {
        MouseEvent {
            x: event.a,
            y: event.b,
            left_button: event.c & 1 == 1,
            middle_button: event.c & 2 == 2,
            right_button: event.c & 4 == 4,
        }
    }

    /// Mouse event trigger
    #[inline]
    pub fn trigger(&self) {
        self.to_event().trigger();
    }
}

/// Escape key
pub const K_ESC: u8 = 0x01;
/// Backspace key
pub const K_BKSP: u8 = 0x0E;
/// Tab key
pub const K_TAB: u8 = 0x0F;
/// Control key
pub const K_CTRL: u8 = 0x1D;
/// Alt key
pub const K_ALT: u8 = 0x38;
/// F1 key
pub const K_F1: u8 = 0x3B;
/// F2 key
pub const K_F2: u8 = 0x3C;
/// F3 key
pub const K_F3: u8 = 0x3D;
/// F4 key
pub const K_F4: u8 = 0x3E;
/// F5 key
pub const K_F5: u8 = 0x3F;
/// F6 key
pub const K_F6: u8 = 0x40;
/// F7 key
pub const K_F7: u8 = 0x41;
/// F8 key
pub const K_F8: u8 = 0x42;
/// F9 key
pub const K_F9: u8 = 0x43;
/// F10 key
pub const K_F10: u8 = 0x44;
/// Home key
pub const K_HOME: u8 = 0x47;
/// Up key
pub const K_UP: u8 = 0x48;
/// Page up key
pub const K_PGUP: u8 = 0x49;
/// Left key
pub const K_LEFT: u8 = 0x4B;
/// Right key
pub const K_RIGHT: u8 = 0x4D;
/// End key
pub const K_END: u8 = 0x4F;
/// Down key
pub const K_DOWN: u8 = 0x50;
/// Page down key
pub const K_PGDN: u8 = 0x51;
/// Delete key
pub const K_DEL: u8 = 0x53;
/// F11 key
pub const K_F11: u8 = 0x57;
/// F12 key
pub const K_F12: u8 = 0x58;

/// A key event (such as a pressed key)
#[derive(Copy, Clone)]
pub struct KeyEvent {
    /// The char of the key
    pub character: char,
    /// The scancode of the key
    pub scancode: u8,
    /// Is the key pressed?
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
        }
    }

    /// Convert from an `Event`
    pub fn from_event(event: Event) -> KeyEvent {
        match char::from_u32(event.a as u32) {
            Some(character) => KeyEvent {
                character: character,
                scancode: event.b as u8,
                pressed: event.c > 0,
            },
            None => KeyEvent {
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

#[derive(Copy, Clone)]
pub struct QuitEvent;

impl QuitEvent {
    pub fn to_event(&self) -> Event {
        Event {
            code: 'q',
            a: 0,
            b: 0,
            c: 0,
        }
    }

    pub fn from_event(_: Event) -> QuitEvent {
        QuitEvent
    }
}
