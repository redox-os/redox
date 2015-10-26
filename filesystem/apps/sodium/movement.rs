use super::*;

impl Editor {
    /// Goto a given position
    pub fn goto(&mut self, (x, y): (usize, usize)) {
        self.cursor_mut().x = x;
        self.cursor_mut().y = y;
    }

    /// Get the position of the next char
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

    /// Get the position of previous char
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

    /// Goto the next char
    pub fn goto_next(&mut self) {
        let p = self.next_pos();
        self.goto(p);
    }
    /// Goto the previous char
    pub fn goto_previous(&mut self) {
        let p = self.previous_pos();
        self.goto(p);
    }

    /// Get the position of the right char
    pub fn right_pos(&self, n: usize) -> (usize, usize) {
        let x = self.x() + n;

        if x > self.text[self.y()].len() {
            (self.text[self.y()].len(), self.y())
        } else {
            (x, self.y())
        }
    }
    /// Goto the right char
    pub fn goto_right(&mut self, n: usize) {
        self.cursor_mut().x = self.right_pos(n).0;
    }

    /// Get the position of the left char
    pub fn left_pos(&self, n: usize) -> (usize, usize) {
        if n <= self.x() {
            (self.x() - n, self.y())
        } else {
            (0, self.y())
        }

    }
    /// Goto the left char
    pub fn goto_left(&mut self, n: usize) {
        self.cursor_mut().x = self.left_pos(n).0;
    }

    /// Get the position of the char above the cursor
    pub fn up_pos(&self, n: usize) -> (usize, usize) {
        if n <= self.y() {
            (self.cursor().x, self.y() - n)
        } else {
            (self.cursor().x, 0)
        }
    }
    /// Go a char up
    pub fn goto_up(&mut self, n: usize) {
        let p = self.up_pos(n);
        self.goto(p);
    }

    /// Get the position under the char
    pub fn down_pos(&self, n: usize) -> (usize, usize) {
        let y = self.y() + n;

        if y >= self.text.len() {
            (self.cursor().x, self.text.len() - 1)
        } else {
            (self.cursor().x, y)
        }
    }

    /// Go down
    pub fn goto_down(&mut self, n: usize) {
        let p = self.down_pos(n);
        self.goto(p);
    }

}
