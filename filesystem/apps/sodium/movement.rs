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
                (self.text[self.y() - 1].len(), self.y() - 1)
            } else {
                (self.x(), self.y())
            }
        } else {
            (self.x() - 1, self.y())
        }
    }

    pub fn goto_next(&mut self) {
        let (x, y) = self.next_pos();
        self.cursor_mut().x = x;
        self.cursor_mut().y = y;
    }
    pub fn goto_previous(&mut self) {
        let (x, y) = self.previous_pos();
        self.cursor_mut().x = x;
        self.cursor_mut().y = y;
    }

    /// Get right pos
    pub fn right_pos(&self, n: usize) -> (usize, usize) {
        let x = self.x() + n;

        if x > self.text[self.y()].len() {
            (self.text[self.y()].len(), self.y())
        } else {
            (x, self.y())
        }
    }
    pub fn goto_right(&mut self, n: usize) {
        self.cursor_mut().x = self.right_pos(n).0;
    }

    /// Get left pos
    pub fn left_pos(&self, n: usize) -> (usize, usize) {
        if n <= self.x() {
            (self.x() - n, self.y())
        } else {
            (0, self.y())
        }

    }
    pub fn goto_left(&mut self, n: usize) {
        self.cursor_mut().x = self.left_pos(n).0;
    }

    /// Get up pos
    pub fn up_pos(&self, n: usize) -> (usize, usize) {
        if n <= self.y() {
            (self.cursor().x, self.y() - n)
        } else {
            (self.cursor().x, 0)
        }
    }
    pub fn goto_up(&mut self, n: usize) {
        let (x, y) = self.up_pos(n);
        self.cursor_mut().x = x;
        self.cursor_mut().y = y;
    }

    /// Get down pos
    pub fn down_pos(&self, n: usize) -> (usize, usize) {
        let y = self.y() + n;

        if y >= self.text.len() {
            (self.cursor().x, self.text.len() - 1)
        } else {
            (self.cursor().x, y)
        }
    }

    pub fn goto_down(&mut self, n: usize) {
        let (x, y) = self.down_pos(n);
        self.cursor_mut().x = x;
        self.cursor_mut().y = y;
    }

}
