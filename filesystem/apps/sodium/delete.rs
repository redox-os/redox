use super::*;

impl Editor {
    /// Delete char
    #[inline]
    pub fn delete(&mut self) {
        let y = self.y();
        let x = self.x();
        if self.text[y].len() == 0 {
            if self.text.len() != 1 {
                self.text.remove(y);
            }
        } else if x < self.text[y].len() {
            self.text[y].remove(x);
        }
    }
}
