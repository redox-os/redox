use redox::*;
use super::*;
use core::iter::FromIterator;

#[derive(Clone, PartialEq, Copy)]
/// The type of the insert mode
pub enum InsertMode {
    /// Append text (after the cursor)
    Append,
    /// Insert text (before the cursor)
    Insert,
    /// Replace text (on the cursor)
    Replace,
}


#[derive(Clone, PartialEq, Copy)]
/// The insert options
pub struct InsertOptions {
    /// The mode type
    pub mode: InsertMode,
}

impl Editor {
    /// Delta x
    pub fn delta(&self) -> usize {
        let (x, y) = self.pos();
        match self.cursor().mode {
            _ if x > self.text[y].len() => {
                debugln!("D is set to 0 because the next char is empty");
                0
            },
            Mode::Primitive(PrimitiveMode::Insert(InsertOptions { mode: InsertMode::Append })) if x == self.text[y].len() => 0,

            Mode::Primitive(PrimitiveMode::Insert(InsertOptions { mode: InsertMode::Append })) => 1,
            _ => 0,
        }
    }

    /// Insert text
    pub fn insert(&mut self, k: Key, InsertOptions { mode: mode }: InsertOptions) {
        let (mut x, mut y) = self.pos();
        match mode {
            InsertMode::Insert | InsertMode::Append => {
                let d = self.delta();
                debugln!("D is {}", d);

                match k {
                    Key::Char('\n') => {
                        let ln = self.text[y].clone();
                        let (slice, _) = ln.as_slices();

                        let first_part = (&slice[..x + d]).clone();
                        let second_part = (&slice[x + d..]).clone();

                        self.text[y] = VecDeque::from_iter(first_part.iter().map(|x| *x));

                        let ind = if self.options.autoindent {
                            self.get_indent(y)
                        } else {
                            VecDeque::new()
                        };
                        let begin = ind.len();

                        self.text.insert(y + 1, VecDeque::from_iter(
                                ind.into_iter().chain(second_part.iter().map(|x| *x))
                        ));

                        self.goto((begin, y + 1));
                    },
                    Key::Backspace => { // Backspace
                        let prev = self.previous();
                        if let Some((x, y)) = prev {
                            //if self.x() != 0 || self.y() != 0 {
                            self.goto((x + d, y));
                            self.delete();
                            //}
                        }
                    },
                    Key::Char(c) => {
                        //debugln!("length is: {}. \n y is: {} \n x is: {} \n x bound is: {}", self.text.len(), y, x, self.text[y].len());
                        self.text[y].insert(x + d, c);

                        match mode {
                            InsertMode::Insert if x + 1 == self.text[y].len() => {                                self.cursor_mut().mode = Mode::Primitive(PrimitiveMode::Insert(InsertOptions {
                                    mode: InsertMode::Append,
                                }));
                                debugln!("Switched to append mode");
                            },
                            _ => {
                                debugln!("No switch x is {}", x);
                            },
                        }
                        
                        let right = self.right(1);
                        self.goto(right);
                    }
                    _ => {},
                }
            },
            InsertMode::Replace => match k {
                Key::Char(c) => {
                    if x == self.text[y].len() {
                        let next = self.next();
                        if let Some(p) = next {
                            self.goto(p);
                            x = self.x();
                            y = self.y();
                        }
                    }

                    if self.text.len() != y {
                        if self.text[y].len() == x {
                            let next = self.next();
                            if let Some(p) = next {
                                self.goto(p);
                            }
                        } else {
                            self.text[y][x] = c;
                        }
                    }
                    let next = self.next();
                    if let Some(p) = next {
                        self.goto(p);
                    }
                },
                _ => {},
            },
        }
    }

    /// Insert a string
    pub fn insert_str(&mut self, txt: String, opt: InsertOptions) {
        for c in txt.chars() {
            self.insert(Key::Char(c), opt);
        }
    }

}
