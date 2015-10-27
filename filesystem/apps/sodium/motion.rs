use super::*;

impl Editor {
    /// Convert an instruction to a motion (new coordinate)
    pub fn to_motion(&mut self, Inst(n, cmd): Inst) -> (usize, usize) {
        use super::Key::*;
        match cmd {
            Char('h') => self.left_pos(n.d()),
            Char('l') => self.right_pos(n.d()),
            Char('j') => self.down_pos(n.d()),
            Char('k') => self.up_pos(n.d()),
            Char('g') => (0, n.or(1) - 1),
            Char('G') => (0, self.text.len() - 1),
            Char('L') => self.ln_end_pos(),
            Char('H') => (0, self.y()),
            _ => (self.x(), self.y()),
        }
    }
}
