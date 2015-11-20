use super::*;
use redox::*;

use orbital::*;

impl Editor {
    /// Redraw the window
    pub fn redraw(&mut self) {
        // TODO: Only draw when relevant for the window
        let (mut pos_x, pos_y) = self.pos();
        // Redraw window
        self.window.set(Color::rgb(25, 25, 25));

        let w = self.window.width();

        pos_x += self.delta();


        if self.options.line_marker {
            self.window.rect(0,
                             (pos_y - self.scroll_y) as isize * 16,
                             w,
                             16,
                             Color::rgb(45, 45, 45));
        }

        self.window.rect(8 * (pos_x - self.scroll_x) as isize,
                         16 * (pos_y - self.scroll_y) as isize,
                         8,
                         16,
                         Color::WHITE);

        let mut string = false;


        for (y, row) in self.text.iter().enumerate() {
            for (x, &c) in row.iter().enumerate() {
                // TODO: Move outta here
                let color = if self.options.highlight {
                    match c {
                        '\'' | '"' => {
                            string = !string;
                            (226, 225, 167) //(167, 222, 156)
                        }
                        _ if string => (226, 225, 167), //(167, 222, 156)
                        '!' |
                        '@' |
                        '#' |
                        '$' |
                        '%' |
                        '^' |
                        '&' |
                        '|' |
                        '*' |
                        '+' |
                        '-' |
                        '/' |
                        ':' |
                        '=' |
                        '<' |
                        '>' => (198, 83, 83), //(228, 190, 175), //(194, 106, 71),
                        '.' | ',' => (241, 213, 226),
                        '(' | ')' | '[' | ']' | '{' | '}' => (164, 212, 125), //(195, 139, 75),
                        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' =>
                            (209, 209, 177),
                        _ => (255, 255, 255),
                    }
                } else {
                    (255, 255, 255)
                };

                let c = if c == '\t' {
                    ' '
                } else {
                    c
                };

                if pos_x == x && pos_y == y {
                    self.window.char(8 * (x - self.scroll_x) as isize,
                                     16 * (y - self.scroll_y) as isize,
                                     c,
                                     Color::rgb(color.0 / 3, color.1 / 3, color.2 / 3));
                } else {
                    self.window.char(8 * (x - self.scroll_x) as isize,
                                     16 * (y - self.scroll_y) as isize,
                                     c,
                                     Color::rgb(color.0, color.1, color.2));
                }
            }
        }

        self.redraw_task = RedrawTask::Null;


        self.redraw_status_bar();
        self.window.sync();
    }

    /// Redraw the status bar
    pub fn redraw_status_bar(&mut self) {
        let h = self.window.height();
        let w = self.window.width();
        let mode = self.cursor().mode;
        self.window.rect(0,
                         h as isize - 18 -
                         {
                             if mode == Mode::Primitive(PrimitiveMode::Prompt) {
                                 18
                             } else {
                                 0
                             }
                         },
                         w,
                         18,
                         Color::rgba(74, 74, 74, 255));

        let sb_mode = self.status_bar.mode.clone();
        status_bar(self, sb_mode, 0, 4);
        let sb_file = self.status_bar.file.clone();
        status_bar(self, sb_file, 1, 4);
        let sb_cmd = self.status_bar.cmd.clone();
        status_bar(self, sb_cmd, 2, 4);
        let sb_msg = self.status_bar.msg.clone();
        status_bar(self, sb_msg, 3, 4);

        for (n, c) in self.prompt.chars().enumerate() {
            self.window.char(n as isize * 8, h as isize - 16 - 1, c, Color::WHITE);
        }

        self.window.sync();
    }
}

fn status_bar(editor: &mut Editor, text: String, a: usize, b: usize) {

    let h = editor.window.height();
    let w = editor.window.width();
    // let y = editor.y();
    let mode = editor.cursor().mode;

    for (n, c) in (if text.len() > w / (8 * b) {
                      text.chars().take(w / (8 * b) - 5).chain(vec!['.'; 3]).collect::<Vec<_>>()
                  } else {
                      text.chars().collect()
                  })
                  .into_iter()
                  .enumerate() {

        editor.window.char(((w * a) / b) as isize + (n as isize * 8),
                           h as isize - 16 - 1 -
                           {
                               if mode == Mode::Primitive(PrimitiveMode::Prompt) {
                                   16 + 1 + 1
                               } else {
                                   0
                               }
                           },
                           c,
                           Color::WHITE);
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
            msg: "Welcome to Sodium!".to_string(),
        }
    }
}
