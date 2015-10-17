use super::*;

impl Editor {
    /// Redraw the window
    pub fn redraw(&mut self) {
        // Redraw window
        self.window.set([255, 255, 255, 255]);

        for (y, row) in self.text.iter().enumerate() {
            for (x, c) in row.iter().enumerate() {
                self.window.char(8 * (y - self.scroll_y) as isize,
                                 16 * (x - self.scroll_x) as isize,
                                 *c,
                                 [128, 128, 128, 255]);
                if self.cursor().x == x && self.cursor().y == y {
                    self.window.char(8 * (y - self.scroll_y) as isize,
                                     16 * (x - self.scroll_x) as isize,
                                     '_',
                                     [128, 128, 128, 255]);
                }
            }
        }
    }
}
