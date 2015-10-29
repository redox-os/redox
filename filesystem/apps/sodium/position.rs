use super::*;

impl Editor {
    /// Get the position of the cursor
    /// (if out of bound, it's the length which is given)
    #[inline]
    pub fn pos(&self) -> (usize, usize) {
        let cursor = self.cursor();
        self.bounded((cursor.x, cursor.y))
    }

    #[inline]
    /// X coordinate
    pub fn x(&self) -> usize {
        self.pos().0
    }
 
    #[inline]
    /// Y coordinate
    pub fn y(&self) -> usize {
        self.pos().1
    }

    /// Convert a position value to a bounded position value
    #[inline]
    pub fn bounded(&self, (x, mut y): (usize, usize)) -> (usize, usize) {

        y = if y > self.text.len() - 1 {
            self.text.len() - 1
        } else {
            y
        };

        if x >= self.text[y].len() {
            (self.text[y].len(), y)
        } else {
            (x, y)
        }
    }
}
