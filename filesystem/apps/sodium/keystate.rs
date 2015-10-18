/// A key state
pub struct KeyState {
    /// Shift pressed
    pub shift: bool,
    /// Ctrl pressed
    pub ctrl: bool,
    /// Alt pressed
    pub alt: bool,
}

impl KeyState {
    pub fn new() -> KeyState {
        KeyState {
            shift: false,
            ctrl: false,
            alt: false,
        }
    }
}
