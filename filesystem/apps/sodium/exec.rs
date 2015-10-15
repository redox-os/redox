use super::*;

impl<I: Iterator<Item = char>> Editor<I> {
    /// Execute a instruction
    pub fn exec(&mut self, inst: Inst) {
        
    }




    /// Feed a char to the editor (as input)
    pub fn feed(&mut self, c: char) {
        match self.cursors[self.current_cursor as usize].mode {
            Mode::Primitive(_) => {
                self.exec(Inst(0, c));
            },
            Mode::Command(_) => {
                self.n = match c {
                    '0' if self.n != 0 => self.n * 10,
                    '1'                => self.n * 10 + 1,
                    '2'                => self.n * 10 + 2,
                    '3'                => self.n * 10 + 3,
                    '4'                => self.n * 10 + 4,
                    '5'                => self.n * 10 + 5,
                    '6'                => self.n * 10 + 6,
                    '7'                => self.n * 10 + 7,
                    '8'                => self.n * 10 + 8,
                    '9'                => self.n * 10 + 9,
                    _                  => {
                        self.exec(Inst(if self.n == 0 { 1 } else { self.n },
                                       c));
                        self.n
                    },
                }

            }
        }
    }
}
