use super::*;
use redox::*;

impl Editor {
    /// Redraw the window
    pub fn redraw(&mut self) {
        // Redraw window
        self.window.set([0, 0, 0, 255]);

        for (y, row) in self.text.iter().enumerate() {
            for (x, c) in row.iter().enumerate() {
                if self.cursor().x == x && self.cursor().y == y {
                    self.window.rect(8 * (x - self.scroll_y) as isize,
                                     16 * (y - self.scroll_x) as isize,
                                     8,
                                     16,
                                     [255, 255, 255, 255]);
                    self.window.char(8 * (x - self.scroll_y) as isize,
                                     16 * (y - self.scroll_x) as isize,
                                     *c,
                                     [0, 0, 0, 255]);
                } else {
                    self.window.char(8 * (x - self.scroll_y) as isize,
                                     16 * (y - self.scroll_x) as isize,
                                     *c,
                                     [255, 255, 255, 255]);
                }
            }
        }
        self.window.sync();
    }
}
