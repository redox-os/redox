use super::*;
use collections::VecDeque;
use redox::*;


#[derive(Clone)]
/// An instruction
pub struct Inst(u16, char);

#[derive(Clone)]
/// The state of the editor
pub struct Editor<I: Iterator<Item = char>> {
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
    /// Number of repeation entered
    pub n: u16,
    /// The input iterator
    pub iter: Option<I>,
}


impl<I: Iterator<Item = char>> Editor<I> {



    /// Create new default state editor
    pub fn new() -> Editor<I> {
        Editor {
            current_cursor: 0,
            cursors: Vec::new(),
            text: VecDeque::new(),
            scroll_x: 0,
            scroll_y: 0,
            n: 0,
            iter: None,
        }
    }
}

