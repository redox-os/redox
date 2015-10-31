use super::*;

impl Editor {
    /// Convert an instruction to a motion (new coordinate)
    pub fn to_motion(&mut self, Inst(n, cmd): Inst) -> Option<(usize, usize)> {
        use super::Key::*;
        match cmd.key {
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

                if let Some(o) = self.next_ocur(ch, n.d()) {
                    Some(o)
                } else {
                    None
                }
            },
            Char(c) => {
                self.status_bar.msg = format!("Motion not defined: '{}'", c);
                self.redraw_status_bar();
                None
            },
            _ => {
                self.status_bar.msg = format!("Motion not defined");
                None
            },
        }
    }
    /// Convert an instruction to a motion (new coordinate) (unbounded)
    pub fn to_motion_unbounded(&mut self, Inst(n, cmd): Inst) -> Option<(isize, isize)> {
        use super::Key::*;
        match cmd.key {
            Char('h') => Some(self.left_unbounded(n.d())),
            Char('l') => Some(self.right_unbounded(n.d())),
            Char('j') => Some(self.down_unbounded(n.d())),
            Char('k') => Some(self.up_unbounded(n.d())),
            Char('g') => Some((0, n.or(1) as isize - 1)),
            Char('G') => Some((0, self.text.len() as isize - 1)),
            Char('L') => Some(to_signed_pos(self.ln_end())),
            Char('H') => Some((0, self.y() as isize)),
            Char('t') => {

                let ch = self.next_char();

                if let Some(o) = self.next_ocur(ch, n.d()) {
                    Some(to_signed_pos(o))
                } else {
                    None
                }
            },
            _ => None,
        }
    }
}
