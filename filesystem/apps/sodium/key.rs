#[derive(Copy, Clone, PartialEq)]
pub enum Key {
    Char(char),
    Alt,
    Shift,
    Ctrl,
    Backspace,
    Escape,
    Left,
    Right,
    Up,
    Down,
    Tab,
    Unknown(u8),
}
