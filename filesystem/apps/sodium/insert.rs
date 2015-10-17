use redox::*;
use super::*;

#[derive(Clone, PartialEq, Copy)]
pub enum InsertMode {
    Append,
    Insert,
    Replace,
}

#[derive(Clone, PartialEq, Copy)]
pub struct InsertOptions {
    pub mode: InsertMode,
}

impl Editor {
    /// Insert text
    pub fn insert(&mut self, c: char) {
        let x = self.x();
        let y = self.y();
        match c {
            '\n' => {
                self.text.insert(y + 1, VecDeque::new());
                self.next();
            },
            '\u{001B}' => { // Escape key
                self.cursor_mut().mode = Mode::Command(CommandMode::Normal);
            },
            '\u{0008}' => {
                self.previous();
                self.delete();
            },
            ' ' => {
                self.next();
            },
            ch => {
                self.text[y].insert(x, ch);
                self.next();
            }
        }
    }
}
