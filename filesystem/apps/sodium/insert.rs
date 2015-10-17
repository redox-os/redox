use redox::*;
use super::*;
use collections::VecDeque;

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
        let cur = self.cursor().clone();
        match c {
            '\n' => {
                self.text.insert(cur.y, VecDeque::new());
            },
            ch => {
                self.text[cur.y].insert(cur.x, ch);
            }
        }
    }
}
