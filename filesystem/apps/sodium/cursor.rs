use Editor;

#[derive(Clone, PartialEq, Hash)]
/// A cursor
pub struct Cursor {
    /// The x coordinate of the cursor
    pub x: u32,
    /// The y coordinate of the cursor
    pub y: u32,
    /// The mode of the cursor
    pub mode: Mode,
    /// The history of the cursor
    pub history: Vec<Inst>,
}

impl Editor {
    /// Get the char under the cursor
    pub fn current(&self) -> char {
        let curs = self.cursor();
        self.text[curs.y][curs.x]
    }

    /// Get the current cursor
    pub fn cursor_mut(&self) -> &Cursor {
        self.cursors.get(self.cur_cursor as usize).unwrap()
    }

    /// Get the current cursor mutable
    pub fn cursor_mut(&mut self) -> &mut Cursor {
        self.cursors.get_mut(self.cur_cursor as usize).unwrap()
    }

    /// Go to next cursor
    pub fn next_cursor(&mut self) {
        self.cursor = (self.cursor + 1) % self.cursors.len();
    }
}
