use super::*;

#[derive(Clone, PartialEq, Copy)]
pub enum InsertMode {
    Append,
    Insert,
    Replace,
}

#[derive(Clone, PartialEq, Copy)]
pub struct InsertOptions {
    mode: InsertMode,
}

impl Editor {
    /// Insert text
    pub fn insert(&mut self, c: char) {
        
    }
}
