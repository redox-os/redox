#[derive(Copy, Clone, PartialEq, Debug)]
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

#[derive(Copy, Clone, PartialEq)]
pub struct Cmd {
    pub key: Key,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

