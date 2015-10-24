use super::*;

impl Editor {
    /// Get pos of next char
    pub fn next_pos(&self) -> (usize, usize) {
        // TODO: Add numerals
        if self.x() == self.text[self.y()].len() {
            if self.y() < self.text.len() - 1 {
                (0, self.y() + 1)
            } else {
                (self.x(), self.y())
            }
        } else {
            (self.x() + 1, self.y())
        }
    }

    /// Get pos of previous char
    pub fn previous_pos(&self) -> (usize, usize) {
        if self.x() == 0 {
            if self.y() > 0 {
                (self.text[self.y()].len(), self.y() - 1)
            } else {
                (self.x(), self.y())
            }
        } else {
            (self.x() - 1, self.y())
        }
    }

    pub fn next(&mut self) {
        let (x, y) = self.next_pos();
        self.cursor_mut().x = x;
        self.cursor_mut().y = y;
    }
    pub fn previous(&mut self) {
        let (x, y) = self.previous_pos();
        self.cursor_mut().x = x;
        self.cursor_mut().y = y;
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
