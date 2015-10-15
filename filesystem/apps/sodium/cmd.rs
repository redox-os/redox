use collections::VecDeque;
// Temporary hack until libredox get hashmaps
use redox::*;

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

/// A mode
#[derive(Clone, PartialEq, Copy, Hash)]
pub enum Mode {
    /// A primitive mode (no repeat, no delimiters, no preprocessing)
    Primitive(PrimitiveMode),
    /// Command mode
    Command(CommandMode),
}

#[derive(Clone, PartialEq, Copy, Hash)]
/// A command mode
pub enum CommandMode {
//    Visual(VisualOptions),
    /// Normal mode
    Normal,
}

#[derive(Clone, PartialEq, Copy, Hash)]
/// A primitive mode
pub enum PrimitiveMode {
    /// Insert mode
    Insert(InsertOptions),
}

#[derive(Clone, PartialEq, Hash)]
/// The state of the editor
pub struct State {
    /// The current cursor
    pub current_cursor: u8,
    /// The cursors
    pub cursors: Vec<Cursor>,
    /// The text (document)
    pub text: VecDeque<VecDeque<char>>,
    /// The x coordinate of the scroll
    pub scroll_x: u32,
    /// The y coordinate of the scroll
    pub scroll_y: u32,
    /// Number of repeation entered
    pub n: u16,
}


impl State {
    fn insert(&mut self, c: char) {

    }
    fn new() -> State {
        State {
            current_cursor: 0,
            cursors: Vec::new(),
            text: VecDeque::new(),
            scroll_x: 0,
            scroll_y: 0,
        }
    }
}

/// A command char
#[derive(Clone, Copy, Hash, PartialEq)]
pub enum CommandChar {
    /// A char
    Char(char),
    /// A wildcard
    Wildcard,
}

/// The editor
pub struct Editor {
    /// The state of the editor
    pub state: State,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            state: State::new(),
        }
    }

    /// Execute a instruction n times
    pub fn exec(&mut self, n: u16, cmd: char) {

        match cmd {

        }
    }
    
    /// Feed a char to the editor (as input)
    fn feed(&mut self, c: char) {
        match self.state.cursors[self.state.current_cursor as usize].mode {
            Mode::Primitive(_) => {
                self.exec(0, c);
            },
            Mode::Command(_) => {
                self.n = match c {
                    '0' if self.n != 0 => self.n * 10,
                    '1'           => self.n * 10 + 1,
                    '2'           => self.n * 10 + 2,
                    '3'           => self.n * 10 + 3,
                    '4'           => self.n * 10 + 4,
                    '5'           => self.n * 10 + 5,
                    '6'           => self.n * 10 + 6,
                    '7'           => self.n * 10 + 7,
                    '8'           => self.n * 10 + 8,
                    '9'           => self.n * 10 + 9,
                    _             => {
                        self.exec(if self.n == 0 { 1 } else { self.n }, c);
                        self.n
                    }
                }

            }
        }
    }
}

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
    pub history: Vec<Unit>,

