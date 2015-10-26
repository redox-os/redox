use super::*;

impl Editor {
    /// Get the x coordinate of the current cursor
    /// (if out of bound, it's the length which is given)
    pub fn x(&self) -> usize {
        let x = self.cursor().x;
        let y = self.cursor().y;
        if y >= self.text.len() {
            0
        } else {
            if x > self.text[y].len() {
                self.text[y].len()
            } else {
                x
            }
        }
    }

    /// Get y coordinate
    pub fn y(&self) -> usize {
        let y = self.cursor().y;
        if y >= self.text.len() {
            self.text.len() - 1
        } else {
            y
        }
    }
}
