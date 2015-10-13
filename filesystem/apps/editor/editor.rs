use redox::*;

#[derive(Copy, Clone)]
pub enum Mode {
    Insert,
    Normal,
}

pub struct Editor {
    url: String,
    file: Option<File>,
    string: String,
    offset: usize,
    scroll_x: isize,
    scroll_y: isize,
}

impl Editor {
    #[inline(never)]
    pub fn new() -> Self {
        Editor {
            url: String::new(),
            file: Option::None,
            string: String::new(),
            offset: 0,
            scroll_x: 0,
            scroll_y: 0,
        }
    }

    fn backspace(&mut self, window: &mut Window) {
         if self.offset > 0 {
             window.set_title(&format!("{}{}{}","Editor (", &self.url, ") Changed"));
             self.string = self.string[0 .. self.offset - 1].to_string() +
                 &self.string[self.offset .. self.string.len()];
             self.offset -= 1;
         }
    }

    fn delete(&mut self, window: &mut Window) {
        if self.offset < self.string.len() {
            window.set_title(&format!("{}{}{}","Editor (", &self.url, ") Changed"));
            self.string = self.string[0 .. self.offset + 1].to_string() +
                &self.string[self.offset + 1 .. self.string.len() - 1];
        }
    }

    fn up(&mut self) {
        let mut new_offset = 0;
        for i in 2..self.offset {
            match self.string.as_bytes()[self.offset - i] {
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

    fn left(&mut self) {
        if self.offset > 0 {
            self.offset -= 1;
        }
    }

    fn right(&mut self) {
        if self.offset < self.string.len() {
            self.offset += 1;
        }
    }

    fn down(&mut self) {
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

    fn reload(&mut self, window: &mut Window) {
        window.set_title(&("Editor (".to_string() + &self.url + ")"));
        self.offset = 0;
        self.scroll_x = 0;
        self.scroll_y = 0;

        match self.file {
            Option::Some(ref mut file) => {
                file.seek(Seek::Start(0));
                let mut vec: Vec<u8> = Vec::new();
                file.read_to_end(&mut vec);
                self.string = unsafe { String::from_utf8_unchecked(vec) };
            },
            Option::None => self.string = String::new(),
        }
    }

    fn save(&mut self, window: &mut Window) {
        match self.file {
            Option::Some(ref mut file) => {
                window.set_title(&("Editor (".to_string() + &self.url + ") Saved"));
                file.seek(Seek::Start(0));
                file.write(&self.string.as_bytes());
                file.sync();
            }
            Option::None => {
                //TODO: Ask for file to save to
                window.set_title(&("Editor (".to_string() + &self.url + ") No Open File"));
            }
        }
    }

    fn draw_content(&mut self, window: &mut Window) {
        let mut redraw = false;

        {
            window.set([255, 255, 255, 255]);

            let scroll_x = self.scroll_x;
            let scroll_y = self.scroll_y;

            let mut offset = 0;

            let mut col = -scroll_x;
            let cols = window.width() as isize / 8;

            let mut row = -scroll_y;
            let rows = window.height() as isize / 16;

            for c in self.string.chars() {
                if offset == self.offset {
                    if col >= 0 && col < cols && row >= 0 && row < rows {
                        window.char(8 * col, 16 * row, '_', [128, 128, 128, 255]);
                    } else {
                        if col < 0 { //Too far to the left
                            self.scroll_x += col;
                        } else if col >= cols { //Too far to the right
                            self.scroll_x += cols - col + 1;
                        }
                        if row < 0 { //Too far up
                            self.scroll_y += row;
                        } else if row >= rows { //Too far down
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
                        window.char(8 * col, 16 * row, c, [0, 0, 0, 255]);
                    }
                    col += 1;
                }

                offset += 1;
            }

            if offset == self.offset {
                if col >= 0 && col < cols && row >= 0 && row < rows {
                    window.char(8 * col, 16 * row, '_', [128, 128, 128, 255]);
                } else {
                    if col < 0 { //Too far to the left
                        self.scroll_x += col;
                    } else if col >= cols { //Too far to the right
                        self.scroll_x += cols - col + 1;
                    }
                    if row < 0 { //Too far up
                        self.scroll_y += row;
                    } else if row >= rows { //Too far down
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
        let mut window = Window::new((rand() % 400 + 50) as isize,
                                     (rand() % 300 + 50) as isize,
                                     576,
                                     400,
                                      &("Editor (".to_string() + url + ")"));

        self.url = url.to_string();
        self.file = Option::Some(File::open(&self.url));

        self.reload(&mut window);
        self.draw_content(&mut window);

        let mut mode = Mode::Normal;

        let mut last_change = String::new();

        while let Option::Some(event) = window.poll() {
            match event.to_option() {
                EventOption::Key(key_event) => {
                    if key_event.pressed {
                        use self::Mode::*;
                        match (mode, key_event.scancode) {
                            (Insert, K_ESC) => {
                                mode = Normal;
                            },
                            (Insert, K_BKSP) => self.backspace(&mut window),
                            (Insert, K_DEL) => self.delete(&mut window),
                            (_, K_F5) => self.reload(&mut window),
                            (_, K_F6) => self.save(&mut window),
                            (_, K_HOME) => self.offset = 0,
                            (_, K_UP) => self.up(),
                            (_, K_LEFT) => self.left(),
                            (_, K_RIGHT) => self.right(),
                            (_, K_END) => self.offset = self.string.len(),
                            (_, K_DOWN) => self.down(),
                            (m, _) => match (m, key_event.character) {
                                (Normal, 'i') => {
                                    mode = Insert;
                                    last_change = self.string.clone();
                                },
                                (Normal, 'h') => self.left(),
                                (Normal, 'l') => self.right(),
                                (Normal, 'k') => self.up(),
                                (Normal, 'j') => self.down(),
                                (Normal, 'G') => self.offset = self.string.len(),
                                (Normal, 'a') => {
                                    self.right();
                                    mode = Insert;
                                    last_change = self.string.clone();
                                },
                                (Normal, 'x') => self.delete(&mut window),
                                (Normal, 'X') => self.backspace(&mut window),
                                (Normal, 'u') => {
                                    self.offset = 0;
                                    ::core::mem::swap(&mut last_change, &mut self.string);
                                },
                                (Insert, '\0') => (),
                                (Insert, _) => {
                                    window.set_title(&format!("{}{}{}","Editor (", &self.url, ") Changed"));
                                    self.string = self.string[0 .. self.offset].to_string() +
                                                  &key_event.character.to_string() +
                                                  &self.string[self.offset .. self.string.len()];
                                    self.offset += 1;
                                },
                                _ => {},
                            }
                        }

                        self.draw_content(&mut window);
                    }
                }
                _ => (),
            }
        }
    }
}

pub fn main() {
    match args().get(1) {
        Option::Some(arg) => Editor::new().main(&arg),
        Option::None => Editor::new().main("none://"),
    }
}
