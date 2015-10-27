use super::*;
use redox::*;

#[derive(Copy, Clone)]
/// An instruction
pub struct Inst(pub Parameter, pub Key);

/// A numeral parameter
#[derive(Copy, Clone)]
pub enum Parameter {
    /// An integer
    Int(usize),
    /// Not given
    Null,
}
impl Parameter {
    /// Either unwrap the Int(n) or fallback to a given value
    #[inline]
    pub fn or(self, fallback: usize) -> usize {
        if let Parameter::Int(n) = self {
            n
        } else {
            fallback
        }
    }
    /// Fallback to one (default)
    #[inline]
    pub fn d(self) -> usize {
        self.or(1)
    }
}

impl Editor {
    /// Get the next char
    pub fn next_char(&mut self) -> char {
        loop {
            if let EventOption::Key(k) = self.window.poll()
                                         .unwrap_or(Event::new())
                                         .to_option() {
                if k.pressed {
                    return k.character;
                }
            }
        }
    }

    /// Get the next instruction
    pub fn next_inst(&mut self) -> Inst {
        let mut n = 0;
        let mut unset = true;

        loop {
            if let EventOption::Key(k) = self.window.poll().unwrap_or(Event::new()).to_option() {
                let c = k.character;
                match c {
                    '\0' => {
                        return Inst(Parameter::Null, match k.scancode {
                            K_ALT => Key::Alt(k.pressed),
                            K_CTRL => Key::Ctrl(k.pressed),
                            K_LEFT_SHIFT | K_RIGHT_SHIFT => Key::Shift(k.pressed),
                            s if k.pressed => match s {
                                K_BKSP => Key::Backspace,
                                K_LEFT => Key::Left,
                                K_RIGHT => Key::Right,
                                K_UP => Key::Up,
                                K_DOWN => Key::Down,
                                K_TAB => Key::Tab,
                                K_ESC => Key::Escape,
                                _ => Key::Unknown(s),
                            },
                            s => Key::Unknown(s),
                        })
                    }
                    _ => if k.pressed {
                        match self.cursor().mode {
                            Mode::Primitive(_) => {
                                return Inst(Parameter::Null, Key::Char(c));
                            },
                            Mode::Command(_) => {
                                n = match c {
                                    '0' => {
                                        unset = false;
                                        n * 10
                                    },
                                    '1' => {
                                        unset = false;
                                        n * 10 + 1
                                    },
                                    '2' => {
                                        unset = false;
                                        n * 10 + 2
                                    },
                                    '3' => {
                                        unset = false;
                                        n * 10 + 3
                                    },
                                    '4' => {
                                        unset = false;
                                        n * 10 + 4
                                    },
                                    '5' => {
                                        unset = false;
                                        n * 10 + 5
                                    },
                                    '6' => {
                                        unset = false;
                                        n * 10 + 6
                                    },
                                    '7' => {
                                        unset = false;
                                        n * 10 + 7
                                    },
                                    '8' => {
                                        unset = false;
                                        n * 10 + 8
                                    },
                                    '9' => {
                                        unset = false;
                                        n * 10 + 9
                                    },
                                    _   => {

                                        return Inst(if unset { Parameter::Null } else { Parameter::Int(n) }, Key::Char(c));
                                    }
                                }
                            }
                        }

                    },
                }
            }
        }

        unreachable!()
    }
}
