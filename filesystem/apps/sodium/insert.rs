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
    /// Insert text
    pub fn insert(&mut self, k: Key, InsertOptions { mode: mode }: InsertOptions) {
        let (mut x, mut y) = self.pos();
        match mode {
            InsertMode::Insert => match k {
                Key::Char('\n') => {
                    let ln = self.text[y].clone();
                    let (slice, _) = ln.as_slices();

                    let first_part = (&slice[..x]).clone();
                    let second_part = (&slice[x..]).clone();

                    self.text[y] = VecDeque::from_iter(first_part.iter().map(|x| *x));

                    let ind = if self.options.autoindent {
                        self.get_indent(y)
                    } else {
                        VecDeque::new()
                    };
                    let begin = ind.len();

                    self.text.insert(y + 1, VecDeque::from_iter(
                            ind.into_iter().chain(second_part.iter().map(|x| *x))));

                    self.goto((begin, y + 1));
                },
                Key::Escape => { // Escape key
                    self.cursor_mut().mode = Mode::Command(CommandMode::Normal);
                },
                Key::Backspace => { // Backspace
                    let prev = self.previous();
                    if let Some(p) = prev {
                        if self.x() != 0 || self.y() != 0 {
                            self.goto(p);
                            self.delete();
                        }
                    }
                },
                Key::Char(c) => {
                    self.text[y].insert(x, c);

                    let right = self.right(1);
                    self.goto(right);
                }
                _ => {},
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
            _ => {},
        }
    }

    /// Insert a string
    pub fn insert_str(&mut self, txt: String, opt: InsertOptions) {
        for c in txt.chars() {
            self.insert(Key::Char(c), opt);
        }
    }

}
