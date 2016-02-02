extern crate orbital;

use std::fs::File;

use orbital::*;
use std::io::*;
use std::env;

pub struct Editor {
    url: String,
    file: Option<File>,
    string: String,
    offset: usize,
    scroll_x: i32,
    scroll_y: i32,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            url: String::new(),
            file: None,
            string: String::new(),
            offset: 0,
            scroll_x: 0,
            scroll_y: 0,
        }
    }

    fn reload(&mut self) {
        self.offset = 0;
        self.scroll_x = 0;
        self.scroll_y = 0;

        match self.file {
            Some(ref mut file) => {
                file.seek(SeekFrom::Start(0));
                let mut string = String::new();
                file.read_to_string(&mut string);
                self.string = string;
            }
            None => self.string = String::new(),
        }
    }

    fn save(&mut self, _: &Window) {
        if self.file.is_none() {
            // let mut save_window = {
            // const WIDTH: usize = 400;
            // const HEIGHT: usize = 200;
            // ConsoleWindow::new((window.x() + (window.width()/2 - WIDTH/2) as isize),
            // (window.y() + (window.height()/2 - HEIGHT/2) as isize),
            // WIDTH,
            // HEIGHT,
            // "Save As")
            // };
            // if let Some(line) = save_window.read() {
            // println!("Create: {}", &line);
            // self.file = File::create(&line);
            // }
            //
        }

        if let Some(ref mut file) = self.file {
            println!("Save: {:?}", file.path());
            println!("  Seek: {:?}", file.seek(SeekFrom::Start(0)));
            println!("  Write: {:?}", file.write(&self.string.as_bytes()));
            println!("  Set length: {:?}", file.set_len(self.string.len()));
            println!("  Sync: {:?}", file.sync_all());
        } else {
            println!("File not open");
        }
    }

    fn draw_content(&mut self, window: &mut Window) {
        let mut redraw = false;

        {
            let gray = Color::rgba(128, 128, 128, 128);
            window.set(Color::WHITE);

            let scroll_x = self.scroll_x;
            let scroll_y = self.scroll_y;

            let mut offset = 0;

            let mut col = -scroll_x;
            let cols = window.width() as i32 / 8;

            let mut row = -scroll_y;
            let rows = window.height() as i32 / 16;

            for c in self.string.chars() {
                if offset == self.offset {
                    if col >= 0 && col < cols && row >= 0 && row < rows {
                        window.rect(8 * col, 16 * row, 8, 16, gray);
                    } else {
                        if col < 0 {
                            // Too far to the left
                            self.scroll_x += col;
                        } else if col >= cols {
                            // Too far to the right
                            self.scroll_x += cols - col + 1;
                        }
                        if row < 0 {
                            // Too far up
                            self.scroll_y += row;
                        } else if row >= rows {
                            // Too far down
                            self.scroll_y += rows - row + 1;
                        }

                        redraw = true;
                    }
                }

                if c == '\n' {
                    col = -scroll_x;
                    row += 1;
                } else if c == '\t' {
                    col += 8 - col % 8;
                } else {
                    if col >= 0 && col < cols && row >= 0 && row < rows {
                        window.char(8 * col, 16 * row, c, Color::BLACK);
                    }
                    col += 1;
                }

                offset += 1;
            }

            if offset == self.offset {
                if col >= 0 && col < cols && row >= 0 && row < rows {
                    window.rect(8 * col, 16 * row, 8, 16, gray);
                } else {
                    if col < 0 {
                        // Too far to the left
                        self.scroll_x += col;
                    } else if col >= cols {
                        // Too far to the right
                        self.scroll_x += cols - col + 1;
                    }
                    if row < 0 {
                        // Too far up
                        self.scroll_y += row;
                    } else if row >= rows {
                        // Too far down
                        self.scroll_y += rows - row + 1;
                    }

                    redraw = true;
                }
            }

            window.sync();
        }

        if redraw {
            self.draw_content(window);
        }
    }

    fn main(&mut self, url: &str) {
        let mut window = Window::new(-1, -1, 576, 400, &("Editor (".to_string() + url + ")"))
                             .unwrap();

        self.url = url.to_string();
        self.file = File::open(&self.url).ok();

        self.reload();
        self.draw_content(&mut window);

        loop {
            for event in window.events() {
                if let EventOption::Key(key_event) = event.to_option() {
                    if key_event.pressed {
                        match key_event.scancode {
                            K_ESC => return,
                            K_BKSP => {
                                if self.offset > 0 {
                                    self.string = self.string[0..self.offset - 1].to_string() +
                                                  &self.string[self.offset..self.string.len()];
                                    self.offset -= 1;
                                }
                            }
                            K_DEL => {
                                if (self.offset) < self.string.len() {
                                    self.string = self.string[0..self.offset].to_string() +
                                                  &self.string[self.offset + 1..self.string.len() - 1];
                                }
                            }
                            K_F5 => self.reload(),
                            K_F6 => self.save(&window),
                            K_HOME => self.offset = 0,
                            K_UP => {
                                let mut new_offset = 0;
                                for i in 2..self.offset {
                                    match self.string.as_bytes()[(self.offset - i) as usize] {
                                        0 => break,
                                        10 => {
                                            new_offset = self.offset - i + 1;
                                            break;
                                        }
                                        _ => (),
                                    }
                                }
                                self.offset = new_offset;
                            }
                            K_LEFT => {
                                if self.offset > 0 {
                                    self.offset -= 1;
                                }
                            }
                            K_RIGHT => {
                                if (self.offset) < self.string.len() {
                                    self.offset += 1;
                                }
                            }
                            K_END => self.offset = self.string.len(),
                            K_DOWN => {
                                let mut new_offset = self.string.len();
                                for i in self.offset..self.string.len() {
                                    match self.string.as_bytes()[i] {
                                        0 => break,
                                        10 => {
                                            new_offset = i + 1;
                                            break;
                                        }
                                        _ => (),
                                    }
                                }
                                self.offset = new_offset;
                            }
                            _ => {
                                match key_event.character {
                                    '\0' => (),
                                    _ => {
                                        self.string = self.string[0..self.offset].to_string() +
                                                      &key_event.character.to_string() +
                                                      &self.string[self.offset..self.string.len()];
                                        self.offset += 1;
                                    }
                                }
                            }
                        }

                        self.draw_content(&mut window);
                    }
                }
                if let EventOption::Quit(_) = event.to_option() {
                    return;
                }
            }
        }
    }
}

fn main() {
    match env::args().nth(1) {
        Some(arg) => Editor::new().main(&arg),
        None => Editor::new().main("none:"),
    }
}
