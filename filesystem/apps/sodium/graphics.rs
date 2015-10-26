use super::*;
use redox::*;

impl Editor {
    /// Redraw the window
    pub fn redraw(&mut self) {
        // TODO: Only draw when relevant for the window
        let x = self.x();
        let y = self.y();
        // Redraw window
        self.window.set(Color::BLACK);

        self.window.rect(8 * (x - self.scroll_y) as isize,
                         16 * (y - self.scroll_x) as isize,
                         8,
                         16,
                         Color::WHITE);

        for (y, row) in self.text.iter().enumerate() {
            for (x, c) in row.iter().enumerate() {
                if self.x() == x && self.y() == y {
                    self.window.char(8 * (x - self.scroll_y) as isize,
                                     16 * (y - self.scroll_x) as isize,
                                     *c,
                                     Color::BLACK);
                } else {
                    self.window.char(8 * (x - self.scroll_y) as isize,
                                     16 * (y - self.scroll_x) as isize,
                                     *c,
                                     Color::WHITE);
                }
            }
        }
        let h = self.window.height();
        let w = self.window.width();
        self.window.rect(0, h as isize - 18, w, 18, Color::rgba(74, 74, 74, 255));

        for (n, c) in (if self.status_bar.mode.len() > w / (8 * 4) {
            self.status_bar.mode.chars().take(w / (8 * 4) - 5).chain(vec!['.', '.', '.']).collect::<Vec<_>>()
        } else {
            self.status_bar.mode.chars().collect()
        }).into_iter().enumerate() {
            self.window.char(n as isize * 8, h as isize - 16 - 1, if c == '\t' { ' ' } else { c }, Color::WHITE);
        }

        self.window.sync();
    }
}

/// The statubar (showing various info about the current state of the editor)
pub struct StatusBar {
    /// The current mode
    pub mode: String,
    /// The cureent char
    pub file: String,
    /// The current command
    pub cmd: String,
    /// A message (such as an error or other info to the user)
    pub msg: String,
}

impl StatusBar {
    /// Create new status bar
    pub fn new() -> Self {
        StatusBar {
            mode: "Normal".to_string(),
            file: String::new(),
            cmd: String::new(),
            msg: String::new(),
        }
    }
}
