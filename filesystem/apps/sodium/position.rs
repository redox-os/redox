use super::*;

impl Editor {
    /// Get the position of the cursor
    /// (if out of bound, it's the length which is given)
    #[inline]
    pub fn pos(&self) -> (usize, usize) {
        let cursor = self.cursor();
        self.bound((cursor.x, cursor.y))
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
    pub fn bound(&self, (x, mut y): (usize, usize)) -> (usize, usize) {


        y = if y >= self.text.len() {
            self.text.len() - 1
        } else {
            y
        };

        let ln = self.text[y].len();
        if x >= ln {
            if ln == 0 {
                (0, y)
            } else {
                (ln - 1, y)
            }
        } else {
            (x, y)
        }
    }

    /// Bound horizontal
    #[inline]
    pub fn bound_hor(&self, (x, y): (usize, usize)) -> (usize, usize) {

        (self.bound((x, y)).0, y)
    }
    /// Bound vertical
    #[inline]
    pub fn bound_ver(&self, (x, mut y): (usize, usize)) -> (usize, usize) {

        // Is this premature optimization? Yes, yes it is!
        y = if y > self.text.len() - 1 {
            self.text.len() - 1
        } else {
            y
        };

        (x, y)
    }
}
