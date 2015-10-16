use super::*;

impl<I: Iterator<Item = char>> Editor<I> {
    /// Go to next char
    pub fn next(&mut self) {
        let die_borrowck_die = self.text.clone();
        let curs = self.cursor_mut();
        if curs.x < die_borrowck_die[curs.y].len() {
            curs.x += 1;
        } else if curs.y < die_borrowck_die.len() {
            curs.x = 0;
            curs.y += 1;
        }
    }

    /// Go right
    pub fn right(&mut self) {
        let text = self.text.clone();
        let curs = self.cursor_mut();
        if text[curs.y].len() < curs.x {
            curs.x += 1;
        }
    }
    /// Go left
    pub fn left(&mut self) {
        let text = self.text.clone();
        let curs = self.cursor_mut();
        if text[curs.y].len() > 0 {
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
        let text = self.text.clone();
        let curs = self.cursor_mut();
        if text.len() < curs.y {
            curs.y += 1;
        }
    }
}
