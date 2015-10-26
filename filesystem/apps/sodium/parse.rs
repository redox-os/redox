use super::*;
use redox::*;

#[derive(Copy, Clone)]
/// An instruction
pub struct Inst(pub Repeat, pub Key);

/// Repeatation
#[derive(Copy, Clone)]
pub enum Repeat {
    /// An integer
    Int(usize),
    /// Not given
    Null,
}
impl Repeat {
    /// Either unwrap the Int(n) or fallback to a given value
    #[inline]
    pub fn or(self, fallback: usize) -> usize {
        if let Repeat::Int(n) = self {
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
    /// Get the next instruction
    pub fn next_inst(&mut self) -> Inst {
        let mut n = 0;
        let mut shifted = false;

        // TODO: Make the switch to normal mode shift more well-coded.
        loop {
            if let EventOption::Key(k) = self.window.poll().unwrap_or(Event::new()).to_option() {
                let c = k.character;
                match c {
                    '\0' => {
                        return Inst(Repeat::Null, match k.scancode {
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
                                return Inst(Repeat::Null, Key::Char(c));
                            },
                            Mode::Command(_) => {
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

                                        return Inst(if n == 0 { Repeat::Null } else { Repeat::Int(n) }, Key::Char(c));
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
