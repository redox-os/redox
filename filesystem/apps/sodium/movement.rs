use super::*;

impl Editor {
    /// Go to next char
    pub fn next(&mut self) {
        if self.x() == self.text[self.y()].len() {
            if self.y() >= self.text.len() {
                self.text.push_back(VecDeque::new())
            }
            if self.y() < self.text.len() - 1 {
                self.cursor_mut().x = 0;
                self.cursor_mut().y += 1;
            }

        } else {
            self.cursor_mut().x += 1;
        }
    }

    /// Go to previous char
    pub fn previous(&mut self) {
        if self.x() == 0 {
            if self.y() > 0 {
                self.cursor_mut().y -= 1;
                self.cursor_mut().x = self.text[self.y()].len();
            }
        } else {
            self.cursor_mut().x -= 1;
        }

    }

    /// Go right
    pub fn right(&mut self, n: usize) {
        let x = self.x() + n;
        let y = self.y();

        let text = self.text.clone();
        let curs = self.cursor_mut();

        curs.x += n;

        if x > text[y].len() {
            curs.x = text[y].len();
        }
    }
    /// Go left
    pub fn left(&mut self, n: usize) {
        let x = self.x();
        let y = self.y();

        let text = self.text.clone();
        let curs = self.cursor_mut();

        if n <= x {
            curs.x = x - n;
        } else {
            curs.x = 0;
        }

    }
    /// Go up
    pub fn up(&mut self, n: usize) {
        let y = self.y();
        let curs = self.cursor_mut();
        if n <= y {
            curs.y -= n;
        } else {
            curs.y = 0;
        }
    }
    /// Go down
    pub fn down(&mut self, n: usize) {
        let x = self.x();
        let y = self.y() + n;

        let text = self.text.clone();
        let curs = self.cursor_mut();

        curs.y += n;

        if y >= text.len() {
            curs.y = text.len() - 1;
        }
    }
}
