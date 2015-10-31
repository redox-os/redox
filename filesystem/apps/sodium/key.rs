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

#[derive(Copy, Clone, PartialEq)]
/// A command, i.e. a key toghether with information on the modifiers.
pub struct Cmd {
    pub key: Key,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

