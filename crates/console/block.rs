use super::{Color, Style};

#[derive(Copy, Clone)]
pub struct Block {
    pub c: char,
    pub fg: Color,
    pub bg: Color,
    pub style: Style
}

impl Block {
    pub fn new() -> Self {
        Block {
            c: ' ',
            fg: Color::ansi(7),
            bg: Color::ansi(0),
            style: Style::Normal
        }
    }
}
