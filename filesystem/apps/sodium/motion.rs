use super::*;

impl Editor {
    /// Convert an instruction to a motion (new coordinate)
    pub fn to_motion(&mut self, Inst(n, cmd): Inst) -> Option<(usize, usize)> {
        use super::Key::*;
        match cmd {
            Char('h') => Some(self.left(n.d())),
            Char('l') => Some(self.right(n.d())),
            Char('j') => Some(self.down(n.d())),
            Char('k') => Some(self.up(n.d())),
            Char('g') => Some((0, n.or(1) - 1)),
            Char('G') => Some((0, self.text.len() - 1)),
            Char('L') => Some(self.ln_end()),
            Char('H') => Some((0, self.y())),
            Char('t') => {

                let ch = self.next_char();

                                                   // ~v~ Optimize (sorry, Knuth)
                if let Some(o) = self.next_ocur(ch, n.d()) {
                    Some(o)
                } else {
                    None
                }
            },
            _ => None,
        }
    }
}
