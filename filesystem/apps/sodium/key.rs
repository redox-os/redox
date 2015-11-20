use orbital::*;

#[derive(Copy, Clone, PartialEq)]
/// A key
pub enum Key {
    Char(char),
    // TODO: Space modifier?
    Backspace,
    Escape,
    Left,
    Right,
    Up,
    Down,
    Tab,
    Null,
    Unknown(u8),
}

impl Key {
    pub fn from_event(k: KeyEvent) -> Key {
        match k.character {
            '\0' => match k.scancode {
                s if k.pressed => match s {
                    K_BKSP => Key::Backspace,
                    K_LEFT => Key::Left,
                    K_RIGHT => Key::Right,
                    K_UP => Key::Up,
                    K_DOWN => Key::Down,
                    K_TAB => Key::Tab,
                    K_ESC => Key::Escape,
                    _ => Key::Unknown(s),

                },
                _ => Key::Null,
            },
            c => Key::Char(c),
        }
    }

    pub fn to_char(self) -> char {
        match self {
            Key::Char(c) => c,
            _ => '\0',
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
/// A command, i.e. a key toghether with information on the modifiers.
pub struct Cmd {
    pub key: Key,
}
