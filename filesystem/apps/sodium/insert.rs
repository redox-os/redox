use Editor;

#[derive(Clone, PartialEq, Copy, Hash)]
pub enum InsertMode {
    Append,
    Insert,
    Replace,
}

#[derive(Clone, PartialEq, Copy, Hash)]
pub struct InsertOptions {
    mode: InsertMode,
}

impl Editor {
    /// Insert text
    pub fn insert(&mut self, c: char) {
        
    }
}
