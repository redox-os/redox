use super::*;

impl Editor {
    /// Remove from a given motion (row based), i.e. if the motion given is to another line, all
    /// the lines from the current one to the one defined by the motion are removed. If the motion
    /// defines a position on the same line, only the characters from the current position to the
    /// motion's position are removed.
    pub fn remove_rb<'a>(&mut self, (x, y): (isize, isize)) {
        if y == self.y() as isize {
            let (x, y) = self.bound((x as usize, y as usize));
            // Single line mode
            let (a, b) = if self.x() > x {
                (x, self.x())
            } else {
                (self.x(), x)
            };
            for _ in a..b {
                self.text[y].remove(a);
            }
        } else {
            let (_, y) = self.bound((x as usize, y as usize));
            // Full line mode
            let (a, b) = if self.y() < y {
                (self.y(), y)
            } else {
                (y, self.y())
            };
            for _ in a..(b + 1) {
                if self.text.len() > 1 {
                    self.text.remove(a);
                } else {
                    self.text[0] = VecDeque::new();
                }
            }
        }
    }
}
