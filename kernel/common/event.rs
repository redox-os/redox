use core::{char, cmp};

use common::scheduler;

/// An optional event
pub enum EventOption {
    /// A mouse event
    Mouse(MouseEvent),
    /// A key event
    Key(KeyEvent),
    /// A display event
    Display(DisplayEvent),
    /// A quit request event
    Quit(QuitEvent),
    /// A unknown event
    Unknown(Event),
    /// No event
    None,
}
pub const DATA_LENGTH: usize = 4;
pub const EVENT_TYPE: isize = 0;
// TODO: make these sequential, no need to use powers of 2
pub const KEYBD_EVENT: isize = 1;
pub const MOUSE_EVENT: isize = 2;
pub const DISP_EVENT: isize = 8;
pub const QUIT_EVENT: isize = 16;
// switched to this because rust doesn't guarantee
// that two structs with the same definition will
// have the same representation when compiled
// unless i misunderstood the rustonomicon
// TODO: double check rustonomicon
pub type EventData = [isize; DATA_LENGTH];
#[derive(Copy,Clone)]
#[repr(packed)]
pub struct Event {
    pub data: EventData,
}

impl Event {
    pub fn new() -> Self {
        Event { data: [0;DATA_LENGTH] }
    }

    pub fn to_option(self) -> EventOption {
        match self.data[EVENT_TYPE as usize] {
            MOUSE_EVENT => EventOption::Mouse(MouseEvent::from_event(self)),
            KEYBD_EVENT => EventOption::Key(KeyEvent::from_event(self)),
            DISP_EVENT  => EventOption::Display(DisplayEvent::from_event(self)),
            QUIT_EVENT  => EventOption::Quit(QuitEvent::from_event(self)),
            0           => EventOption::None,
            _           => EventOption::Unknown(self),
        }
    }

    // Trigger an event
    pub fn trigger(&self) {
        let event = *self;
        unsafe {
            let reenable = scheduler::start_no_ints();
            (*::events_ptr).push(event);
            scheduler::end_no_ints(reenable);
        }
    }
}

#[derive(Copy, Clone)]
pub struct MouseEvent {
    /// The x coordinate
    pub x: isize,
    /// The y coordinate
    pub y: isize,
    /// Is the left button pressed?
    pub left_button: bool,
    /// Is the middle button pressed?
    pub middle_button: bool,
    /// Is the right button pressed?
    pub right_button: bool,
}

impl MouseEvent {
    /// Convert to an `Event`
    pub fn to_event(&self) -> Event {
        let button_state = (self.left_button as isize)       | 
                           (self.right_button as isize) << 1 |
                           (self.middle_button as isize) << 2;
        Event { 
            data: [ MOUSE_EVENT as isize, self.x, self.y, button_state ],
        }
    }

    pub fn from_event(event: Event) -> MouseEvent {
        let button_info = event.data[3];
        let left_button = button_info & 1 == 1;
        let right_button = button_info & 2 == 2;
        let middle_button = button_info & 4 == 4;
        MouseEvent {
            x: event.data[1],
            y: event.data[2],
            left_button: left_button,
            right_button: right_button,
            middle_button: middle_button,
        }
    }

    #[inline]
    pub fn trigger(&self) {
        self.to_event().trigger()
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

#[derive(Copy, Clone)]
pub struct KeyEvent {
    pub character: char,
    pub scancode: u8,
    pub pressed: bool,
}

impl KeyEvent {
    pub fn to_event(&self) -> Event {
        Event {
            data: [ KEYBD_EVENT , 
                    self.character as isize, 
                    self.scancode as isize, 
                    self.pressed as isize]
        }
    }

    pub fn from_event(event: Event) -> KeyEvent {
        let ch = char::from_u32(event.data[1] as u32);
        match ch {
            Some(character) => KeyEvent {
                character: character,
                scancode: event.data[2] as u8,
                pressed: event.data[3] > 0,
            },
            None => KeyEvent {
                character: '\0',
                scancode: event.data[2] as u8,
                pressed: event.data[3] > 0,
            },
        }
    }

    #[inline]
    pub fn trigger(&self) {
        self.to_event().trigger();
    }
}

#[derive(Copy,Clone)]
pub struct DisplayEvent {
    pub restricted: bool,
}

impl DisplayEvent {
    pub fn to_event(&self) -> Event {
        Event {
            data: [ DISP_EVENT, 
                      self.restricted as isize, 
                      0,
                      0]
        }
    }

    pub fn from_event(event: Event) -> DisplayEvent {
        DisplayEvent {
            restricted: event.data[1] > 0,
        }
    }

    #[inline]
    pub fn trigger(&self) {
        self.to_event().trigger();
    }
}

// TODO: does this belong in kernel space?
#[derive(Copy, Clone)]
pub struct QuitEvent;

impl QuitEvent {
    pub fn to_event(&self) -> Event {
        Event {
            data: [ QUIT_EVENT, 0, 0, 0 ]
        }
    }

    pub fn from_event(event: Event) -> QuitEvent {
        QuitEvent
    }
}
