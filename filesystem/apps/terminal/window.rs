
use orbital::*;

/// A console char
pub struct ConsoleChar {
    /// The char
    character: char,
    /// The color
    color: Color,
}

/// A console window
pub struct ConsoleWindow {
    /// The window
    pub window: Box<Window>,
    /// The char buffer
    pub output: Vec<ConsoleChar>,
    /// Previous commands
    pub history: Vec<String>,
    /// History index
    pub history_i: u32,
    /// Offset
    pub offset: usize,
    /// Scroll distance x
    pub scroll_x: i32,
    /// Scroll distance y
    pub scroll_y: i32,
    /// Wrap the text, if true
    pub wrap: bool,
}

impl ConsoleWindow {
    /// Create a new console window
    pub fn new(x: i32, y: i32, w: u32, h: u32, title: &str) -> Box<Self> {
        Box::new(ConsoleWindow {
            window: Window::new(x, y, w, h, title).unwrap(),
            output: Vec::new(),
            history: vec!["".to_string()],
            history_i: 0,
            offset: 0,
            scroll_x: 0,
            scroll_y: 0,
            wrap: true,
        })
    }

    /// Set the window title
    pub fn set_title(&mut self, title: &str) {
        // TODO THIS IS A HACK, should use self.window.setTitle(title);
        self.window = Window::new(self.window.x(),
                                  self.window.y(),
                                  self.window.width(),
                                  self.window.height(),
                                  title)
                          .unwrap();
    }

    /// Print to the window
    pub fn print(&mut self, string: &str, color: Color) {
        for c in string.chars() {
            self.output.push(ConsoleChar {
                character: c,
                color: color,
            });
        }
        self.sync();
    }

    /// Read input
    pub fn read(&mut self) -> Option<String> {
        loop {
            for event in self.window.events() {
                match event.to_option() {
                    EventOption::Key(key_event) => {
                        if key_event.pressed {
                            match key_event.scancode {
                                K_BKSP => {
                                    if self.offset > 0 {
                                        self.history[self.history_i as usize] =
                                            self.history[self.history_i as usize][0..self.offset - 1]
                                                .to_string() +
                                            &self.history[self.history_i as usize][self.offset..];
                                        self.offset -= 1;
                                    }
                                }
                                K_DEL => {
                                    if (self.offset) < self.history[self.history_i as usize].len() {
                                        self.history[self.history_i as usize] =
                                    self.history[self.history_i as usize][0..self.offset].to_string() +
                                    &self.history[self.history_i as usize][self.offset + 1..self.history[self.history_i as usize].len() - 1];
                                    }
                                }
                                K_HOME => self.offset = 0,
                                K_UP => {
                                    if self.history_i as usize + 1 < self.history.len() {
                                        self.history_i += 1;
                                    }
                                    self.offset = self.history[self.history_i as usize].len();
                                }
                                K_LEFT => {
                                    if self.offset > 0 {
                                        self.offset -= 1;
                                    }
                                }
                                K_RIGHT => {
                                    if (self.offset) < self.history[self.history_i as usize].len() {
                                        self.offset += 1;
                                    }
                                }
                                K_END => self.offset = self.history[self.history_i as usize].len(),
                                K_DOWN => {
                                    if self.history_i > 0 {
                                        self.history_i -= 1;
                                    }
                                    self.offset = self.history[self.history_i as usize].len();
                                }
                                _ => {
                                    match key_event.character {
                                        '\x00' => (),
                                        '\n' => {
                                            let command = self.history[self.history_i as usize].clone();
                                            self.offset = 0;
                                            self.history_i = 0;
                                            if !self.history[0].is_empty() {
                                                self.history.insert(0, "".to_string());
                                            }
                                            while self.history.len() > 1000 {
                                                self.history.pop();
                                            }
                                            self.print(&command, Color::WHITE);
                                            self.print("\n", Color::WHITE);
                                            return Some(command);
                                        }
                                        '\x1B' => (),
                                        _ => {
                                            self.history[self.history_i as usize] =
                                                self.history[self.history_i as usize][0..self.offset]
                                                    .to_string() +
                                                &key_event.character.to_string() +
                                                &self.history[self.history_i as usize][self.offset..];
                                            self.offset += 1;
                                        }
                                    }
                                }
                            }
                            self.sync();
                        }
                    }
                    EventOption::Quit(_quit_event) => return None,
                    _ => (),
                }
            }
        }
    }

    /// Redraw the window
    pub fn sync(&mut self) {
        let scroll_x = self.scroll_x;
        let scroll_y = self.scroll_y;

        let mut col = -scroll_x;
        let cols = self.window.width() as i32 / 8;
        let mut row = -scroll_y;
        let rows = self.window.height() as i32 / 16;

        {
            self.window.set(Color::BLACK);

            for c in self.output.iter() {
                if self.wrap && col >= cols {
                    col = -scroll_x;
                    row += 1;
                }

                if c.character == '\n' {
                    col = -scroll_x;
                    row += 1;
                } else if c.character == '\t' {
                    col += 8 - col % 8;
                } else {
                    if col >= 0 && col < cols && row >= 0 && row < rows {
                        self.window.char(8 * col, 16 * row, c.character, c.color);
                    }
                    col += 1;
                }
            }

            let mut i = 0;
            for c in self.history[self.history_i as usize].chars() {
                if self.wrap && col >= cols {
                    col = -scroll_x;
                    row += 1;
                }

                if self.offset == i && col >= 0 && col < cols && row >= 0 && row < rows {
                    self.window.char(8 * col, 16 * row, '_', Color::WHITE);
                }

                if c == '\n' {
                    col = -scroll_x;
                    row += 1;
                } else if c == '\t' {
                    col += 8 - col % 8;
                } else {
                    if col >= 0 && col < cols && row >= 0 && row < rows {
                        self.window.char(8 * col, 16 * row, c, Color::WHITE);
                    }
                    col += 1;
                }

                i += 1;
            }

            if self.wrap && col >= cols {
                col = -scroll_x;
                row += 1;
            }

            if self.offset == i && col >= 0 && col < cols && row >= 0 && row < rows {
                self.window.char(8 * col, 16 * row, '_', Color::WHITE);
            }
        }

        self.window.sync();

        if row >= rows {
            self.scroll_y += row - rows + 1;

            self.sync();
        }
    }
}
