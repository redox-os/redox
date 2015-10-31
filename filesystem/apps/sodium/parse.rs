use super::*;
use redox::*;

#[derive(Copy, Clone)]
/// An instruction
pub struct Inst(pub Parameter, pub Cmd);

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
                    self.status_bar.cmd.push(k.character);
                    self.redraw_status_bar();
                    return k.character;
                }
            }
        }
    }

    /// Get the next instruction
    pub fn next_inst(&mut self) -> Inst {
        let mut n = 0;
        let mut unset = true;

        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;

        let mut key = Key::Null;
        self.status_bar.cmd = String::new();

//        self.status_bar.cmd = String::new();
        loop {
            if let EventOption::Key(k) = self.window.poll().unwrap_or(Event::new()).to_option() {

                let c = k.character;
                match c {
                    '\0' => {
                        // HERES THE BUG! It returns a modifier (which it shouldnt)
                        match k.scancode {
                            K_ALT => alt = k.pressed,
                            K_CTRL => ctrl = k.pressed,
                            K_LEFT_SHIFT | K_RIGHT_SHIFT => shift = k.pressed,
                            s if k.pressed => key = match s {
                                K_BKSP => Key::Backspace,
                                K_LEFT => Key::Left,
                                K_RIGHT => Key::Right,
                                K_UP => Key::Up,
                                K_DOWN => Key::Down,
                                K_TAB => Key::Tab,
                                K_ESC => Key::Escape,
                                _ => Key::Unknown(s),
                            },
                            s => key = Key::Unknown(s),
                        }
                    }
                    _ => if k.pressed {
                        self.status_bar.cmd.push(c);
                        self.redraw_status_bar();

                        match self.cursor().mode {
                            Mode::Primitive(_) => {
                                key = Key::Char(c);
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

                                        key = Key::Char(c);
                                        n
                                        //return Inst(if unset { Parameter::Null } else { Parameter::Int(n) }, Key::Char(c));
                                    }
                                };
//                                self.status_bar.cmd.push(c);
                            }
                        }

                    },
                }
            }
            if key != Key::Null {
                return Inst(if unset { Parameter::Null } else { Parameter::Int(n) }, {
                    Cmd {
                        key: key,
                        ctrl: ctrl,
                        alt: alt,
                        shift: shift,
                    }
                });
            }
        }

    }
}
