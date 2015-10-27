use super::*;

impl Editor {
    /// Get the x coordinate of the current cursor
    /// (if out of bound, it's the length which is given)
    #[inline]
    pub fn x(&self) -> usize {
        let x = self.cursor().x;
        let y = self.y();
        if x >= self.text[y].len() {
            self.text[y].len()
        } else {
            x
        }
    }

    /// Get y coordinate
    #[inline]
    pub fn y(&self) -> usize {
        let y = self.cursor().y;
        if y > self.text.len() - 1 {
            self.text.len() - 1
        } else {
            y
        }
    }
}
