use super::*;

impl Editor {
    /// Delete a character
    #[inline]
    pub fn delete(&mut self) {
        let (x, y) = self.pos();
        if self.text[y].is_empty() {
            if self.text.len() != 1 {
                self.text.remove(y);
            }
        } else if x < self.text[y].len() {
            self.text[y].remove(x);
        }
    }
}
