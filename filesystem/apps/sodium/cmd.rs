use collections::VecDeque;
// Temporary hack until libredox get hashmaps
use redox::*;

/// A temporary, very slow replacement for HashMaps, until redox::collections is finish.
pub struct HashMapTmp<K, V> {
    data: Vec<(K, V)>,
}
impl<K: PartialEq, V> HashMapTmp<K, V> {
    pub fn get(&self, key: &K) -> Option<&V> {
        match self.data.iter().find(|(k, _)| {
            k == *key
        }) {
            Some((k, ref v)) => Some(v),
            None => None
        }
    }
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.data.iter().find(|(k, _)| {
            k == *key
        }) {
            Some((k, ref mut v)) => Some(v),
            None => None
        }
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let old = self.get_mut(&key);

        match old {
            Some(v) => {
                *v = value;
                Some(*v)
            },
            None => {
                self.data.push((key, value));
                None
            },
        }
    }
}

type Map<K, V> = HashMapTmp<K, V>;

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
pub enum Unit {
    /// Single [repeated] instruction
    Inst(u16, char),
    /// Multiple instructions
    Block(u16, Vec<Unit>),
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
#[derive(Clone, Copy, Hash, PartialEq)]
pub enum CommandChar {
    /// A char
    Char(char),
    /// A wildcard
    Wildcard,
}

#[derive(Clone, PartialEq, Hash)]
/// The editor
pub struct Editor<'a, I: Iterator<Item = Unit>> {
    /// The state of the editor
    pub state: State,
    /// The commands
    pub commands: Map<Mode,
                      Map<CommandChar,
                          Box<FnOnce(u16, &'a mut State, &'a mut I, char)>>>,
}

impl<'a, I: Iterator<Item = Unit>> Editor<'a, I> {
    pub fn new() -> Self {
        let mut commands = Map::new();
        commands.insert(Mode::Primitive(PrimitiveMode::Insert), {
            let mut hm = Map::new();
            hm.insert(CommandChar::Wildcard, |_, state, iter, c| {
                state.insert(c);
            });
            hm
        });
        Editor {
            state: State::new(),
            commands: commands,
        }
    }

    pub fn exec(&mut self, cmd: &Unit) {
        let mut commands = self.commands.get(&self.mode).unwrap();

        match *cmd {
            Unit::Inst(n, c) => {
                commands.get(&c).call((n, &mut self.state, self.cmd.iter(), c));
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
}

#[derive(Clone, PartialEq, Hash)]
/// An iterator over units
pub struct UnitIterator<'a, I: Iterator<Item = char> + 'a> {
    /// The iterator over the chars
    char_iter: &'a mut I,
    /// The state
    state: &'a mut State,
}

impl<'a, I: Iterator<Item = char>> Iterator for UnitIterator<'a, I> {
    type Item = Unit;

    fn next(&mut self) -> Option<Unit> {
        match self.cursors[self.cursor as usize].mode {
            Mode::Primitive(_) => Unit::Inst(1, match self.char_iter.next() {
                Some(c) => c,
                None => return None,
            }),
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
                    Some(Unit::Block(n, self.collect()))
                } else if let Some(ch) = self.char_iter.next() {
                    Some(Unit::Inst(n, ch))
                } else {
                    None
                }
            }
        }
    }
}

pub trait ToUnitIterator: Iterator<Item = char> {
    /// Create a iterator of the unit given by the chars
    fn unit_iter<'a>(&'a self, state: &'a mut State) -> UnitIterator<'a, Self>;
}

impl<I: Iterator<Item = char>> ToUnitIterator for I {
    fn unit_iter<'a>(&'a self, state: &'a mut State) -> UnitIterator<'a, I> {
        UnitIterator {
            char_iter: &mut *self,
            state: state,
        }
    }
}
