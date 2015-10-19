use super::*;

impl Editor {
    /// Delete char
    pub fn delete(&mut self) {
        let y = self.y();
        let x = self.x();
        if self.text[y].len() == 1 {
            self.text.remove(y);
        } else {
            self.text[y].remove(x);
        }
    }
}
