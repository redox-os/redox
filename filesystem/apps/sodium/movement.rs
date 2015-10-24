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

    /// Get right pos
    pub fn right_pos(&self, n: usize) -> (usize, usize) {
        let x = self.x() + n;
        let y = self.y();

        if x > self.text[y].len() {
            (self.text[y].len(), y)
        } else {
            (x, y)
        }
    }
    pub fn right(&mut self, n: usize) {
        self.cursor_mut().x = self.right_pos(n).0;
    }

    /// Get left pos
    pub fn left_pos(&self, n: usize) -> (usize, usize) {
        let x = self.x();
        let y = self.y();

        if n <= x {
            (x - n, self.y())
        } else {
            (0, self.y())
        }

    }
    pub fn left(&mut self, n: usize) {
        self.cursor_mut().x = self.left_pos(n).0;
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
