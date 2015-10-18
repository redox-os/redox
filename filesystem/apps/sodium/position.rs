use super::*;

impl Editor {
    /// Get the x coordinate of the current cursor
    /// (if out of bound, it's the length which is given)
    pub fn x(&self) -> usize {
        let x = self.cursor().x;
        let y = self.cursor().y;
        if x > self.text[y].len() {
            self.text[y].len()
        } else {
            x
        }
    }

    /// Get y coordinate
    pub fn y(&self) -> usize {
        self.cursor().y
    }
}
