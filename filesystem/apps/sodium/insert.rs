use redox::*;
use super::*;
use core::iter::FromIterator;

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
                let ln = self.text[y].clone();
                let (slice, _) = ln.as_slices();

                let first_part = (&slice[..x]).clone();
                let second_part = (&slice[x..]).clone();

                self.text[y] = VecDeque::from_iter(first_part.iter().map(|x| *x));
                self.text.insert(y + 1, VecDeque::from_iter(second_part.iter().map(|x| *x)));

                self.next();
            },
            '\u{001B}' => { // Escape key
                self.cursor_mut().mode = Mode::Command(CommandMode::Normal);
            },
            '\u{0008}' => { // Backspace
                if self.x() != 0 || self.y() != 0 {
                    self.previous();
                    self.delete();
                }
            },
            ch => {
                self.text[y].insert(x, ch);
                self.next();
            }
        }
    }
}
