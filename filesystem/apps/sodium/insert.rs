use redox::*;
use super::*;
use core::iter::FromIterator;

#[derive(Clone, PartialEq, Copy)]
/// The type of the insert mode
pub enum InsertMode {
    Append,
    Insert,
    Replace,
}

#[derive(Clone, PartialEq, Copy)]
/// The insert options
pub struct InsertOptions {
    /// The mode type
    pub mode: InsertMode,
}

impl Editor {
    /// Insert text
    pub fn insert(&mut self, c: Key) {
        let x = self.x();
        let y = self.y();
        match c {
            Key::Char('\n') => {
                let ln = self.text[y].clone();
                let (slice, _) = ln.as_slices();

                let first_part = (&slice[..x]).clone();
                let second_part = (&slice[x..]).clone();

                self.text[y] = VecDeque::from_iter(first_part.iter().map(|x| *x));
                self.text.insert(y + 1, VecDeque::from_iter(second_part.iter().map(|x| *x)));

                self.next();
            },
            Key::Escape => { // Escape key
                self.cursor_mut().mode = Mode::Command(CommandMode::Normal);
            },
            Key::Backspace => { // Backspace
                if self.x() != 0 || self.y() != 0 {
                    self.previous();
                    self.delete();
                }
            },
            Key::Char(ch) => {
                self.text[y].insert(x, ch);
                self.next();
            }
            _ => {},
        }
    }
}
