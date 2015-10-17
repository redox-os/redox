use super::*;

impl Editor {
    /// Go to next char
    pub fn next(&mut self) {
        if self.x() == self.text[self.y()].len() {
            if self.y() >= self.text.len() {
                self.text.push_back(VecDeque::new())
            }
            self.cursor_mut().x = 0;
            self.cursor_mut().y += 1;

        } else {
            self.cursor_mut().x += 1;
        }
    }

    /// Go to previous char
    pub fn previous(&mut self) {
        if self.x() == 0 {
            if self.y() > 0 {
                self.cursor_mut().y -= 1;
                self.cursor_mut().x = self.text[self.y() - 1].len();
            }
        } else {
            self.cursor_mut().y -= 1;
        }

    }

    /// Go right
    pub fn right(&mut self) {
        let x = self.x();
        let y = self.y();
        let text = self.text.clone();
        let curs = self.cursor_mut();
        if x < text[y].len() {
            curs.x += 1;
        }
    }
    /// Go left
    pub fn left(&mut self) {
        let x = self.x();
        let y = self.y();
        let text = self.text.clone();
        let curs = self.cursor_mut();
        if x > 0 {
            curs.x -= 1;
        }
    }
    /// Go up
    pub fn up(&mut self) {
        let y = self.y();
        let curs = self.cursor_mut();
        if y > 0 {
            curs.y -= 1;
        }
    }
    /// Go down
    pub fn down(&mut self) {
        let x = self.x();
        let y = self.y();
        let text = self.text.clone();
        let curs = self.cursor_mut();
        if y + 1 < text.len() {
            curs.y += 1;
        }
    }
}
