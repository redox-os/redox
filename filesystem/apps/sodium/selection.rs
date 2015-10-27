use super::*;
use redox::*;

impl Editor {
    /// Remove from a given motion (row based)
    pub fn remove_rb<'a>(&mut self, (x, y): (usize, usize)) {
        if y == self.y() {
            // Single line mode
            let (a, b) = if self.x() < x {
                (self.x(), x)
            } else {
                (x, self.x())
            };
            for _ in a..b {
                self.text[y].remove(a);
            }
        } else {
            // Full line mode
            let (a, b) = if self.y() < y {
                (self.y(), y)
            } else {
                (y, self.y())
            };
            for _ in a..(b + 1) {
                if self.text.len() > 1 {
                    self.text.remove(a);
                }
            }
        }
    }
}
