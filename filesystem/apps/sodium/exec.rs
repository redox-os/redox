use super::*;

struct InstructionIterator<'a, I: 'a> {
    editor: &'a mut Editor<I>,
}

impl<'a, I: Iterator<Item = char>> Iterator for InstructionIterator<'a, I> {
    type Item = Inst;
    
    fn next(&mut self) -> Option<Inst> {
        let mut n = 0;

        match self.editor.iter {
            Some(i) => {
                let last;
                for c in i {
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
            },
            None => None,
        }
    }
}

impl<I: Iterator<Item = char>> Editor<I> {
    pub fn instr_iter<'a>(&'a mut self) -> InstructionIterator<'a, I> {
        InstructionIterator {
            editor: self,
        }
    }
}
impl<I: Iterator<Item = char>> Editor<I> {
    /// Execute a instruction
    pub fn exec(&mut self, inst: Inst) {
        
    }

}
