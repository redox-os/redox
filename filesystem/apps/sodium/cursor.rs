use super::*;

#[derive(Clone)]
/// A cursor, i.e. a state defining a mode, and a position. The cursor does not define the content
/// of the current file.
pub struct Cursor {
    /// The x coordinate of the cursor
    pub x: usize,
    /// The y coordinate of the cursor
    pub y: usize,
    /// The mode of the cursor
    pub mode: Mode,
}

impl Cursor {
    /// Create a new default cursor
    pub fn new() -> Cursor {
        Cursor {
            x: 0,
            y: 0,
            mode: Mode::Command(CommandMode::Normal),
        }
    }
}

impl Editor {
    /// Get the character under the cursor
    #[inline]
    pub fn current(&self) -> char {
        let (x, y) = self.pos();
        self.text[y][x]
    }

    /// Get the current cursor
    #[inline]
    pub fn cursor(&self) -> &Cursor {
        self.cursors.get(self.current_cursor as usize).unwrap()
    }

    /// Get the current cursor mutably
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
