use redox::collections::VecDeque;
use redox::collections::HashMap;

#[derive(Clone, Copy, Hash)]
pub enum InsertMode {
    Append,
    Insert,
    Replace,
}

#[derive(Clone, Copy, Hash)]
pub struct InsertOptions {
    mode: InsertMode,
}

/// A mode
#[derive(Clone, Copy, Hash)]
pub enum Mode {
    /// A primitive mode (no repeat, no delimiters, no preprocessing)
    Primitive(PrimitiveMode),
    /// Command mode
    Command(CommandMode),
}

/// A command mode
pub enum Command {
//    Visual(VisualOptions),
    /// Normal mode
    Normal,
}

/// A primitive mode
pub enum PrimitiveMode {
    /// Insert mode
    Insert(InsertOptions),
}

#[derive(Clone, Hash)]
pub enum Unit {
    /// Single [repeated] instruction
    Inst(u16, char),
    /// Multiple instructions
    Block(u16, Vec<Unit>),
}

#[derive(Clone, Hash)]
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
}

impl State {
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
pub enum CommandChar {
    /// A char
    Char(char),
    /// A wildcard
    Wildcard,
}

#[derive(Clone, Hash)]
/// The editor
pub struct Editor<I: Iterator<Item = Unit>> {
    /// The state of the editor
    pub state: State,
    /// The commands
    pub commands: HashMap<Mode,
                          HashMap<CommandChar,
                                  FnMut<<(u16, &mut State, &mut I, c)>>>>,
}

impl<I: Iterator<Item = Unit> Editor<I> {
    pub fn new() -> Self {
        let mut commands = HashMap::new();
        commands.insert(Mode::Primitive(PrimitiveMode::Insert), {
            let mut hm = HashMap::new();
            hm.insert(CommandChar::Wildcard, |_, state, iter, c| {
                editor.insert(c);
            });
            hm
        });
        Editor {
            state: State::new(),
            commands: commands,
        }
    }

    pub fn exec(&mut self, cmd: &Unit) {
        let mut commands = self.commands.get(self.mode).unwrap();

        match *cmd {
            Unit::Single(n, c) => {
                commands.get(c).call((n, &mut self.state, self.cmd.iter(), c));
            },
            Unit::Block(n, units) => {
                for _ in 1..n {
                    for &i in units {
                        self.exec(i);
                    }
                }
            },
        }
    }
}

#[derive(Clone, Hash)]
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
}

#[derive(Clone, Hash)]
/// An iterator over units
pub struct UnitIterator<'a, I: Iterator<Item = char>> {
    /// The iterator over the chars
    char_iter: &'a mut I,
    /// The state
    state: &'a mut State,
}

impl<'a, I: Iterator<Item = char>> Iterator for UnitIterator<'a, I> {
    type Item = Unit;

    fn next(&mut self) -> Unit {
        match self.cursors[self.cursor as usize].mode {
            Mode::Primitive(_) => Unit::Single(1, self.char_iter.next().unwrap()),
            Mode::Command(_) => {
                let mut ch = self.first().unwrap_or('\0');
                let mut n = 1;

                let mut unset = true;
                for c in self.char_iter {
                    n = match c {
                        '0' if n != 0 => n * 10,
                        '1'           => n * 10 + 1,
                        '2'           => n * 10 + 2,
                        '3'           => n * 10 + 3,
                        '4'           => n * 10 + 4,
                        '5'           => n * 10 + 5,
                        '6'           => n * 10 + 6,
                        '7'           => n * 10 + 7,
                        '8'           => n * 10 + 8,
                        '9'           => n * 10 + 9,
                        _             => {
                            ch = c;
                            break;
                        },
                    };

                    if unset {
                        unset = false;
                        n     = 0;
                    }
                }


                if ch == '(' {
                    let mut level = 0;
                    *self = self.take_while(|c| {
                        level = match c {
                            '(' => level + 1,
                            ')' => level - 1,
                            ';' => 0,
                        }
                    }).skip(1).reverse().skip(1).reverse().unit_iter();
                    Unit::Block(n, self.collect())
                } else if let Some(ch) = self.char_iter.next() {
                    Unit::Inst(n, ch)
                }
            }
        }
    }
}

impl<I: Iterator<Item = char>> I {
    /// Create a iterator of the unit given by the chars
    pub fn unit_iter<'a>(&'a self, state: &'a mut State) -> UnitIterator<'a, I> {
        UnitIterator {
            char_iter: &mut *self,
            state: state,
        }
    }
}
