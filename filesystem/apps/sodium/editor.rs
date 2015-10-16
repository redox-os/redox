use super::*;
use collections::VecDeque;
use redox::*;


#[derive(Copy, Clone)]
/// An instruction
pub struct Inst(pub u16, pub char);

#[derive(Clone)]
/// The state of the editor
pub struct Editor {
    /// The current cursor
    pub current_cursor: u8,
    /// The cursors
    pub cursors: Vec<Cursor>,
    /// The text (document)
    pub text: VecDeque<VecDeque<char>>,
    /// The x coordinate of the scroll
    pub scroll_x: usize,
    /// The y coordinate of the scroll
    pub scroll_y: usize,
}


impl Editor {



    /// Create new default state editor
    pub fn new() -> Editor {
        Editor {
            current_cursor: 0,
            cursors: Vec::new(),
            text: VecDeque::new(),
            scroll_x: 0,
            scroll_y: 0,
        }
    }
}

