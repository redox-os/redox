use super::*;

impl Editor {
    /// Convert an instruction to a motion (new coordinate)
    pub fn to_motion(&mut self, Inst(n, cmd): Inst) -> (usize, usize) {
        use super::Key::*;
        match cmd {
            Char('h') => self.left(n.d()),
            Char('l') => self.right(n.d()),
            Char('j') => self.down(n.d()),
            Char('k') => self.up(n.d()),
            Char('g') => (0, n.or(1) - 1),
            Char('G') => (0, self.text.len() - 1),
            Char('L') => self.ln_end(),
            Char('H') => (0, self.y()),
            Char('t') => {
                let ch = self.next_char();

                // TODO!
                (self.x(), self.y())
            },
            _ => (self.x(), self.y()),
        }
    }
}
