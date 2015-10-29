use super::*;

// TODO! Clear up naming!

impl Editor {
    /// Goto a given position
    #[inline]
    pub fn goto(&mut self, (x, y): (usize, usize)) {
        self.cursor_mut().x = x;
        self.cursor_mut().y = y;
    }

    /// Get the previous position
    pub fn previous(&self) -> Option<(usize, usize)> {
        self.before(self.pos())
    }
    /// Get the next position
    pub fn next(&self) -> Option<(usize, usize)> {
        self.after(self.pos())
    }

    /// Get position of char after a given char
    #[inline]
    pub fn after(&self, (x, y): (usize, usize)) -> Option<(usize, usize)> {

        if x == self.text[y].len() {
            if y < self.text.len() - 1 {
                Some((0, y + 1))
            } else {
                None //(x, y)
            }
        } else {
            Some((x + 1, y))
        }
    }

    /// Get the position of the char before a given char's position
    pub fn before(&self, (x, y): (usize, usize)) -> Option<(usize, usize)> {
        if x == 0 {
            if y > 0 {
                Some((self.text[y - 1].len(), y - 1))
            } else {
                None //(x, y)
            }
        } else {
            Some((x - 1, y))
        }
    }

    /// Get the position of the right char
    #[inline]
    pub fn right(&self, n: usize) -> (usize, usize) {
        (self.x() + n, self.y())
    }

    /// Get the position of the left char
    #[inline]
    pub fn left(&self, n: usize) -> (usize, usize) {
        if n <= self.x() {
            (self.x() - n, self.y())
        } else {
            (0, self.y())
        }
    }

    /// Get the position of the char above the cursor
    #[inline]
    pub fn up(&self, n: usize) -> (usize, usize) {
        if n <= self.y() {
            (self.cursor().x, self.y() - n)
        } else {
            (self.cursor().x, 0)
        }
    }

    /// Get the position under the char
    #[inline]
    pub fn down(&self, n: usize) -> (usize, usize) {
        (self.cursor().x, self.y() + n)
    }

    /// Get the position of the end of the line
    #[inline]
    pub fn ln_end(&self) -> (usize, usize) {
        (self.text[self.y()].len(), self.y())
    }

    /// Get next ocurrence of a given charecter
    #[inline]
    pub fn next_ocur(&self, c: char, n: usize) -> Option<(usize, usize)> {
        let mut dn = 0;

        let mut pos = self.after(self.pos());
        loop {

            match pos {
                None => return None,
                Some(mut p) => {
                    p = self.bounded(p);

                    if self.text[p.1][p.0] == c {
                        dn += 1;
                        if dn == n {
                            return Some(p);
                        }
                    }

                    pos = self.after(p);

                },
            }


        }
    }
}
