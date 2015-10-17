use super::*;
use redox::*;
use core::marker::Sized;

pub struct InstructionIterator<'a, I: 'a> {
    pub editor: &'a mut Editor,
    pub iter: &'a mut I,
}

impl<'a, I: Iterator<Item = EventOption>> Iterator for InstructionIterator<'a, I> {
    type Item = Inst;

    fn next(&mut self) -> Option<Inst> {
        let mut n = 0;

        let mut last = '\0';
        while let Some(EventOption::Key(k)) = self.iter.next() {
            if k.pressed {
                let c = k.character;
                match self.editor.cursor().mode {
                    Mode::Primitive(_) => {
                        Inst(0, c);
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
                                last = c;
                                break;
                            }
                        }
                    }
                }
            }
        }
        Some(Inst(if n == 0 { 1 } else { n }, last))
    }
}

pub trait ToInstructionIterator
          where Self: Sized {
    fn inst_iter<'a>(&'a mut self, editor: &'a mut Editor) -> InstructionIterator<'a, Self>;
}

impl<I> ToInstructionIterator for I
        where I: Iterator<Item = EventOption> + Sized {
    fn inst_iter<'a>(&'a mut self, editor: &'a mut Editor) -> InstructionIterator<'a, Self> {
        InstructionIterator {
            editor: editor,
            iter: self,
        }
    }
}
