use super::*;
use redox::*;

pub struct InstructionIterator<'a, I: 'a> {
    pub editor: &'a Editor,
    pub iter: &'a mut I,
}

impl<'a, I: Iterator<Item = EventOption>> Iterator for InstructionIterator<'a, I> {
    type Item = Inst;
    
    fn next(&mut self) -> Option<Inst> {
        let mut n = 0;

        let mut last = '\0';
        for e in self.iter {
            match e {
                EventOption::Key(k) if k.pressed => {
                    let c = k.character;
                    match self.editor.cursors[self.editor.current_cursor as usize].mode {
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
                                },
                            }

                        }
                    }
        }

        Some(Inst(if n == 0 { 1 } else { n }, last))
    }
}

trait ToInstructionIterator {
    fn inst_iter<'a>(&'a mut self, editor: &'a Editor) -> InstructionIterator<'a, Self>;
}

impl ToInstructionIterator for Iterator<Item = EventOption> {
    fn inst_iter<'a>(&'a mut self, editor: &'a Editor) -> InstructionIterator<'a, Self> {
        InstructionIterator {
            editor: self,
            iter: self,
        }
    }
}
impl Editor {
    /// Execute a instruction
    pub fn exec(&mut self, inst: Inst) {
        
    }

}
