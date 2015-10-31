use super::*;

// TODO! Clear up naming!

impl Editor {
    /// Goto a given position. Does not automatically bound.
    #[inline]
    pub fn goto(&mut self, (x, y): (usize, usize)) {
        self.cursor_mut().x = x;
        self.cursor_mut().y = y;
    }

    /// Get the previous position, i.e. the position before the cursor (*not* left to the cursor)
    #[inline]
    pub fn previous(&self) -> Option<(usize, usize)> {
        self.before(self.pos())
    }
    /// Get the next position, i.e. the position after the cursor (*not* right to the cursor)
    #[inline]
    pub fn next(&self) -> Option<(usize, usize)> {
        self.after(self.pos())
    }

    /// Get position after a given position, i.e. a generalisation of .next()
    #[inline]
    pub fn after(&self, (x, y): (usize, usize)) -> Option<(usize, usize)> {

        if y < self.text.len() - 1 {
            if x == self.text[y].len() - 1 {
                Some((0, y + 1))
            } else {
                Some((x + 1, y))
            }
        } else if x == self.text[y].len() - 1 {
            None
        } else {
            Some((x + 1, y))
        }
    }

    /// Get the position before a given position, i.e. a generalisation .before()
    #[inline]
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

    /// Get the position of the character right to the cursor (horizontally bounded)
    #[inline]
    pub fn right(&self, n: usize) -> (usize, usize) {
        self.bound_hor((self.x() + n, self.y()))
    }
    /// Get the position of the character right to the cursor (unbounded)
    #[inline]
    pub fn right_unbounded(&self, n: usize) -> (isize, isize) {
        ((self.x() + n) as isize, self.y() as isize)
    }

    /// Get the position of the character left to the cursor (horizontally bounded)
    #[inline]
    pub fn left(&self, n: usize) -> (usize, usize) {
        if n <= self.x() {
            (self.x() - n, self.y())
        } else {
            (0, self.y())
        }
    }
    /// Get the position of the character left to the cursor (unbounded)
    #[inline]
    pub fn left_unbounded(&self, n: usize) -> (isize, isize) {
        (self.x() as isize - n as isize, self.y() as isize)
    }

    /// Get the position of the character above the cursor (vertically bounded)
    #[inline]
    pub fn up(&self, n: usize) -> (usize, usize) {
        if n <= self.y() {
            (self.cursor().x, self.y() - n)
        } else {
            (self.cursor().x, 0)
        }
    }
    /// Get the position of the character above the cursor (unbounded)
    #[inline]
    pub fn up_unbounded(&self, n: usize) -> (isize, isize) {
        (self.cursor().x as isize, self.y() as isize - n as isize)
    }

    /// Get the position of the character under the cursor (vertically bounded)
    #[inline]
    pub fn down(&self, n: usize) -> (usize, usize) {
        self.bound_ver((self.cursor().x, self.y() + n))
    }
    /// Get the position of the character above the cursor (unbounded)
    #[inline]
    pub fn down_unbounded(&self, n: usize) -> (isize, isize) {
        (self.cursor().x as isize, self.y() as isize + n as isize)
    }

    /// Get the position of the end of the line
    #[inline]
    pub fn ln_end(&self) -> (usize, usize) {
        (self.text[self.y()].len(), self.y())
    }

    /// Get n'th next ocurrence of a given charecter (relatively to the cursor)
    pub fn next_ocur(&self, c: char, n: usize) -> Option<(usize, usize)> {
        let mut dn = 0;

        let mut pos = self.after(self.pos());
        loop {

            match pos {
                None => return None,
                Some(mut p) => {
                    p = self.bound(p);

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
    /// Get n'th previous ocurrence of a given charecter (relatively to the cursor)
    pub fn previous_ocur(&self, c: char, n: usize) -> Option<(usize, usize)> {
        let mut dn = 0;

        let mut pos = self.before(self.pos());
        loop {

            match pos {
                None => return None,
                Some(mut p) => {
                    p = self.bound(p);

                    if self.text[p.1][p.0] == c {
                        dn += 1;
                        if dn == n {
                            return Some(p);
                        }
                    }

                    pos = self.before(p);

                },
            }


        }
    }


}

