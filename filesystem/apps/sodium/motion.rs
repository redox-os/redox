use super::*;

impl Editor {
    /// Convert an instruction to a motion (new coordinate)
    pub fn to_motion(&self, Inst(n, cmd): Inst) -> (usize, usize) {
        use super::Key::*;
        match cmd {
            Char('h') => self.left_pos(n),
            Char('l') => self.right_pos(n),
            Char('j') => self.down_pos(n),
            Char('k') => self.up_pos(n),
            Char('$') => self.ln_end_pos(),
            Char('0') => (0, self.y()),
            _ => (self.x(), self.y()),
        }
    }
}
