use redox::*;
use super::*;

#[derive(Clone)]
/// A cursor
pub struct Cursor {
    /// The x coordinate of the cursor
    pub x: usize,
    /// The y coordinate of the cursor
    pub y: usize,
    /// The mode of the cursor
    pub mode: Mode,
    /// The history of the cursor
    pub history: Vec<Inst>,
}

impl Cursor {
    /// New default cursor
    pub fn new() -> Cursor {
        Cursor {
            x: 0,
            y: 0,
            mode: Mode::Command(CommandMode::Normal),
            history: Vec::new(),
        }
    }
}

impl Editor {
    /// Get the char under the cursor
    #[inline]
    pub fn current(&self) -> char {
        let curs = self.cursor();
        self.text[curs.y][curs.x]
    }

    /// Get the current cursor
    #[inline]
    pub fn cursor(&self) -> &Cursor {
        self.cursors.get(self.current_cursor as usize).unwrap()
    }

    /// Get the current cursor mutable
    #[inline]
    pub fn cursor_mut(&mut self) -> &mut Cursor {
        self.cursors.get_mut(self.current_cursor as usize).unwrap()
    }

    /// Go to next cursor
    #[inline]
    pub fn next_cursor(&mut self) {
        self.current_cursor = (self.current_cursor + 1) % (self.cursors.len() as u8);
    }
}
