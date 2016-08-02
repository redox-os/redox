use core::{char, mem, slice};
use core::ops::{Deref, DerefMut};

pub const EVENT_NONE: i64 = 0;
pub const EVENT_MOUSE: i64 = 1;
pub const EVENT_KEY: i64 = 2;
pub const EVENT_QUIT: i64 = 3;

/// An optional event
#[derive(Copy, Clone, Debug)]
pub enum EventOption {
    /// A mouse event
    Mouse(MouseEvent),
    /// A key event
    Key(KeyEvent),
    /// A quit request event
    Quit(QuitEvent),
    /// An unknown event
    Unknown(Event),
    /// No event
    None,
}

/// An event
#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct Event {
    pub code: i64,
    pub a: i64,
    pub b: i64,
    pub c: i64,
}

impl Event {
    /// Create a null event
    pub fn new() -> Event {
        Event {
            code: 0,
            a: 0,
            b: 0,
            c: 0,
        }
    }

    /// Convert the event ot an optional event
    // TODO: Consider doing this via a From trait.
    pub fn to_option(self) -> EventOption {
        match self.code {
            EVENT_NONE => EventOption::None,
            EVENT_MOUSE => EventOption::Mouse(MouseEvent::from_event(self)),
            EVENT_KEY => EventOption::Key(KeyEvent::from_event(self)),
            EVENT_QUIT => EventOption::Quit(QuitEvent::from_event(self)),
            _ => EventOption::Unknown(self),
        }
    }
}

impl Deref for Event {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self as *const Event as *const u8, mem::size_of::<Event>()) as &[u8]
        }
    }
}

impl DerefMut for Event {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self as *mut Event as *mut u8, mem::size_of::<Event>()) as &mut [u8]
        }
    }
}

/// A event related to the mouse
#[derive(Copy, Clone, Debug)]
pub struct MouseEvent {
    /// The x coordinate of the mouse
    pub x: i32,
    /// The y coordinate of the mouse
    pub y: i32,
    /// Was the left button pressed?
    pub left_button: bool,
    /// Was the middle button pressed?
    pub middle_button: bool,
    /// Was the right button pressed?
    pub right_button: bool,
}

impl MouseEvent {
    /// Convert to an `Event`
    pub fn to_event(&self) -> Event {
        Event {
            code: EVENT_MOUSE,
            a: self.x as i64,
            b: self.y as i64,
            c: self.left_button as i64 | (self.middle_button as i64) << 1 |
               (self.right_button as i64) << 2,
        }
    }

    /// Convert an `Event` to a `MouseEvent`
    pub fn from_event(event: Event) -> MouseEvent {
        MouseEvent {
            x: event.a as i32,
            y: event.b as i32,
            left_button: event.c & 1 == 1,
            middle_button: event.c & 2 == 2,
            right_button: event.c & 4 == 4,
        }
    }
}

pub const K_A: u8 = 0x1E;
pub const K_B: u8 = 0x30;
pub const K_C: u8 = 0x2E;
pub const K_D: u8 = 0x20;
pub const K_E: u8 = 0x12;
pub const K_F: u8 = 0x21;
pub const K_G: u8 = 0x22;
pub const K_H: u8 = 0x23;
pub const K_I: u8 = 0x17;
pub const K_J: u8 = 0x24;
pub const K_K: u8 = 0x25;
pub const K_L: u8 = 0x26;
pub const K_M: u8 = 0x32;
pub const K_N: u8 = 0x31;
pub const K_O: u8 = 0x18;
pub const K_P: u8 = 0x19;
pub const K_Q: u8 = 0x10;
pub const K_R: u8 = 0x13;
pub const K_S: u8 = 0x1F;
pub const K_T: u8 = 0x14;
pub const K_U: u8 = 0x16;
pub const K_V: u8 = 0x2F;
pub const K_W: u8 = 0x11;
pub const K_X: u8 = 0x2D;
pub const K_Y: u8 = 0x15;
pub const K_Z: u8 = 0x2C;
pub const K_0: u8 = 0x0B;
pub const K_1: u8 = 0x02;
pub const K_2: u8 = 0x03;
pub const K_3: u8 = 0x04;
pub const K_4: u8 = 0x05;
pub const K_5: u8 = 0x06;
pub const K_6: u8 = 0x07;
pub const K_7: u8 = 0x08;
pub const K_8: u8 = 0x09;
pub const K_9: u8 = 0x0A;

/// Tick/tilde key
pub const K_TICK: u8 = 0x29;
/// Minus/underline key
pub const K_MINUS: u8 = 0x0C;
/// Equals/plus key
pub const K_EQUALS: u8 = 0x0D;
/// Backslash/pipe key
pub const K_BACKSLASH: u8 = 0x2B;
/// Bracket open key
pub const K_BRACE_OPEN: u8 = 0x1A;
/// Bracket close key
pub const K_BRACE_CLOSE: u8 = 0x1B;
/// Semicolon key
pub const K_SEMICOLON: u8 = 0x27;
/// Quote key
pub const K_QUOTE: u8 = 0x28;
/// Comma key
pub const K_COMMA: u8 = 0x33;
/// Period key
pub const K_PERIOD: u8 = 0x34;
/// Slash key
pub const K_SLASH: u8 = 0x35;
/// Backspace key
pub const K_BKSP: u8 = 0x0E;
/// Space key
pub const K_SPACE: u8 = 0x39;
/// Tab key
pub const K_TAB: u8 = 0x0F;
/// Capslock
pub const K_CAPS: u8 = 0x3A;
/// Left shift
pub const K_LEFT_SHIFT: u8 = 0x2A;
/// Right shift
pub const K_RIGHT_SHIFT: u8 = 0x36;
/// Control key
pub const K_CTRL: u8 = 0x1D;
/// Alt key
pub const K_ALT: u8 = 0x38;
/// Enter key
pub const K_ENTER: u8 = 0x1C;
/// Escape key
pub const K_ESC: u8 = 0x01;
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
#[derive(Copy, Clone, Debug)]
pub struct KeyEvent {
    /// The charecter of the key
    pub character: char,
    /// The scancode of the key
    pub scancode: u8,
    /// Was it pressed?
    pub pressed: bool,
}

impl KeyEvent {
    /// Convert to an `Event`
    pub fn to_event(&self) -> Event {
        Event {
            code: EVENT_KEY,
            a: self.character as i64,
            b: self.scancode as i64,
            c: self.pressed as i64,
        }
    }

    /// Convert from an `Event`
    pub fn from_event(event: Event) -> KeyEvent {
        KeyEvent {
            character: char::from_u32(event.a as u32).unwrap_or('\0'),
            scancode: event.b as u8,
            pressed: event.c > 0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct QuitEvent;

impl QuitEvent {
    pub fn to_event(&self) -> Event {
        Event {
            code: EVENT_QUIT,
            a: 0,
            b: 0,
            c: 0,
        }
    }

    pub fn from_event(_: Event) -> QuitEvent {
        QuitEvent
    }
}
