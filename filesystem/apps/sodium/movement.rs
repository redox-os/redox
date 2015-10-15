use super::*;

impl<I: Iterator<Item = char>> Editor<I> {
    /// Go to next char
    pub fn next(&mut self) {
        let curs = self.cursor_mut();
        if curs.x < self.text[curs.y].len() {
            curs.x += 1;
        } else if curs.y < self.text.len() {
            curs.x = 0;
            curs.y += 1;
        }
    }

    /// Go right
    pub fn right(&mut self) {
        let curs = self.cursor_mut();
        if self.text[curs.y].len() < curs.x {
            curs.x += 1;
        }
    }
    /// Go left
    pub fn left(&mut self) {
        let curs = self.cursor_mut();
        if self.text[curs.y].len() > 0 {
            curs.x -= 1;
        }
    }
    /// Go up
    pub fn up(&mut self) {
        let curs = self.cursor_mut();
        if curs.y > 0 {
            curs.y -= 1;
        }
    }
    /// Go down
    pub fn down(&mut self) {
        let curs = self.cursor_mut();
        if self.text.len() < curs.y {
            curs.y += 1;
        }
    }
}
