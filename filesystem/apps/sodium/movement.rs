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
    pub fn previous(&self, n: usize) -> Option<(usize, usize)> {
        self.before(n, self.pos())
    }
    /// Get the next position, i.e. the position after the cursor (*not* right to the cursor)
    #[inline]
    pub fn next(&self, n: usize) -> Option<(usize, usize)> {
        self.after(n, self.pos())
    }

    /// Get position after a given position, i.e. a generalisation of .next()
    #[inline]
    pub fn after(&self, n: usize, (x, y): (usize, usize)) -> Option<(usize, usize)> {
        // FIXME: Bound?
        // TODO: Return enum

       if x + n < self.text[y].len() {

         //  debugln!("IF1 x: {}, n: {}, len: {}", x, n, self.text[y].len());
           Some((x + n, y))
       } else {
        //   debugln!("ELSE x: {}, n: {}, len: {}", x, n, self.text[y].len());
           if y + 1 >= self.text.len() {
       //        debugln!("RETURN NONE (IF2) x: {}, n: {}, len: {}", x, n, self.text[y].len());
               None
           } else {
      //         debugln!("ELSE2 x: {}, n: {}, len: {}", x, n, self.text[y].len());
               let mut mv = n + x - self.text[y].len();
               let mut ry = y + 1;
     //          debugln!("ELSE2 mv: {}, ry: {}", mv, ry);

               loop {
                   if mv < self.text[ry].len() {
    //                   debugln!("LOOP:IF1 mv: {}, ry: {}", mv, ry);
                       return Some((mv, ry));
                   } else {
   //                    debugln!("LOOP:ELSE1 mv: {}, ry: {}", mv, ry);
                       if ry + 1 < self.text.len() {
  //                         debugln!("LOOP:IF2 mv: {}, ry: {}", mv, ry);
                           mv -= self.text[ry].len();
                           ry += 1;
                       } else {
 //                          debugln!("LOOP:ELSE2 mv: {}, ry: {}", mv, ry);
                           return None;
                       }
                   }
               }

           }
       }
    }

    /// Get the position before a given position, i.e. a generalisation .before()
    #[inline]
    pub fn before(&self, n: usize, (x, y): (usize, usize)) -> Option<(usize, usize)> {
        if x >= n {
//            debugln!("IF1 x: {}, n: {}, len: {}", x, n, self.text[y].len());
            Some((x - n, y))
        } else {
//            debugln!("ELSE x: {}, n: {}, len: {}", x, n, self.text[y].len());
            if y == 0 {
//                debugln!("RETURN NONE (IF2) x: {}, n: {}, len: {}", x, n, self.text[y].len());
                None
            } else {
//                debugln!("ELSE2 x: {}, n: {}, len: {}", x, n, self.text[y].len());
                let mut mv = n - x;
                let mut ry = y - 1;
//                debugln!("ELSE2 mv: {}, ry: {}", mv, ry);

                loop {
                    if mv <= self.text[ry].len() {
//                        debugln!("LOOP:IF1 mv: {}, ry: {}", mv, ry);
                        return Some((x - mv, ry));
                    } else {
//                        debugln!("LOOP:ELSE1 mv: {}, ry: {}", mv, ry);
                        if ry > 0 && mv >= self.text[ry].len() {
 //                           debugln!("LOOP:IF2 mv: {}, ry: {}", mv, ry);
                            mv -= self.text[ry].len();
                            ry -= 1;
                        } else if ry == 0 {
//                            debugln!("LOOP:ELSE2 mv: {}, ry: {}", mv, ry);
                            return None;
                        }
                    }
                }
            }
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

        let mut pos = self.after(1, self.pos());
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

                    pos = self.after(1, p);

                },
            }


        }
    }
    /// Get n'th previous ocurrence of a given charecter (relatively to the cursor)
    pub fn previous_ocur(&self, c: char, n: usize) -> Option<(usize, usize)> {
        let mut dn = 0;

        let mut pos = self.before(1, self.pos());
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

                    pos = self.before(1, p);

                },
            }


        }
    }


}

