use super::*;
use redox::*;

impl Editor {
    /// Redraw the window
    pub fn redraw(&mut self) {
        // TODO: Only draw when relevant for the window
        let x = self.x();
        let y = self.y();
        // Redraw window
        self.window.set(Color::rgb(25, 25, 25));

        self.window.rect(8 * (x - self.scroll_y) as isize,
                         16 * (y - self.scroll_x) as isize,
                         8,
                         16,
                         Color::WHITE);

        let mut string = false;

        for (y, row) in self.text.iter().enumerate() {
            for (x, &c) in row.iter().enumerate() {
                // TODO: Move outta here
                let color = match c {
                    '\'' | '"' => {
                        string = !string;
                        (226, 225, 167) //(167, 222, 156)
                    },
                    _ if string => (226, 225, 167), //(167, 222, 156)
                    '!' | '@' | '#' | '$' | '%' | '^' | '&' | '|' | '*' | '+' | '-' | '/' | ':' | '=' | '<' | '>' => (198, 83, 83), //(228, 190, 175), //(194, 106, 71),
                    '.' | ',' => (241, 213, 226),
                    '(' | ')' | '[' | ']' | '{' | '}' => (164, 212, 125), //(195, 139, 75),
                    '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => (209, 209, 177),
                    _ => (255, 255, 255),
                };

                let c = if c == '\t' { ' ' } else { c };

                if self.x() == x && self.y() == y {
                    self.window.char(8 * (x - self.scroll_y) as isize,
                                     16 * (y - self.scroll_x) as isize,
                                     c,
                                     Color::rgb(color.0 / 3, color.1 / 3, color.2 / 3));
                } else {
                    self.window.char(8 * (x - self.scroll_y) as isize,
                                     16 * (y - self.scroll_x) as isize,
                                     c,
                                     Color::rgb(color.0, color.1, color.2));
                }
            }
        }
        let h = self.window.height();
        let w = self.window.width();
        let mode = self.cursor().mode;

        self.window.rect(0, h as isize - 18 - {
            if mode == Mode::Primitive(PrimitiveMode::Prompt) {
                18
            } else {
                0
            }
        }, w, 18, Color::rgba(74, 74, 74, 255));

        for (n, c) in (if self.status_bar.mode.len() > w / (8 * 4) {
            self.status_bar.mode.chars().take(w / (8 * 4) - 5).chain(vec!['.', '.', '.']).collect::<Vec<_>>()
        } else {
            self.status_bar.mode.chars().collect()
        }).into_iter().enumerate() {

            self.window.char(n as isize * 8, h as isize - 16 - 1 - {
                if mode == Mode::Primitive(PrimitiveMode::Prompt) {
                    16 + 1
                } else {
                    0
                }
            }, c, Color::WHITE);
        }

        for (n, c) in self.prompt.chars().enumerate() {
            self.window.char(n as isize * 8, h as isize - 16 - 1, c, Color::WHITE);
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
