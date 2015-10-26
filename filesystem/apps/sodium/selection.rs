use super::*;
use redox::*;

impl Editor {
    /// Get from a given motion (row based)
    pub fn get_rb<'a>(&self, (x, y): (usize, usize)) -> Vec<&[char]> {
        if y == self.y() {
            // Single line mode
            vec![&self.get_ln(y)[self.x()..x]]
        } else {
            let mut lines = Vec::new();
            // Full line mode
            for ln in self.y()..y {
                lines.push(self.get_ln(ln));
            }
            lines
        }
    }
}
