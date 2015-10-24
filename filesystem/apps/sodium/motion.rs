use super::*;

impl Editor {
    pub fn to_motion(&self, Inst(n, cmd): Inst) -> (usize, usize) {
        match cmd {
            'h' => self.left_pos(n),
            'l' => self.right_pos(n),
            'j' => self.down_pos(n),
            'k' => self.up_pos(n),
            _ => (0, 0),
        }
    }
}
