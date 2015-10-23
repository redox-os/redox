// TODO: Refactor using a matrix for performance

use redox::*;

mod cmd;


#[derive(Copy, Clone, PartialEq, Eq)]
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
            file: None,
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
            self.string = self.string[0 .. self.offset].to_string() +
                &self.string[self.offset + 1 .. self.string.len()];
        }
    }

    // TODO: Add methods for multiple movements
    fn up(&mut self) {
        if self.cur() == '\n' || self.cur() == '\0' {
            self.left();
            while self.cur() != '\n' &&
                  self.cur() != '\0' &&
                  self.offset >= 1 {
                self.left();
            }
        } else {
            let x = self.get_x(); //- if self.cur() == '\n' { 1 } else { 0 };

            while self.cur() != '\n' &&
                  self.offset >= 1 {
                self.left();
            }
            self.right();
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
            for _ in 1..x {
                if self.cur() != '\n' {
                    self.right();
                } else {
                    break;
                }
            }
        }
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
        let x = self.get_x(); //- if self.cur() == '\n' { 1 } else { 0 };
        let original_c = self.cur();

        while self.offset < self.string.len() &&
              self.cur() != '\n' &&
              self.cur() != '\0' {
            self.right();
        }
        self.right();

        if original_c == '\n' {
            while self.cur() != '\n' &&
                  self.cur() != '\0' &&
                  self.offset < self.string.len() {
                self.right();
            }
        } else {
            for _ in 1..x {
                if self.cur() != '\n' {
                    self.right();
                } else {
                    break;
                }
            }
        }
    }

    fn cur(&self) -> char {
        self.string.chars().nth(self.offset).unwrap_or('\0')
    }

    fn insert(&mut self, c: char, window: &mut Window) {
        let ind = if c == '\n' {
            let mut mov = 0;

            for _ in 0..self.get_x() {
                self.left();
                mov += 1;
            }

            let mut ind = String::new();
            while (self.cur() == ' ' ||
                  self.cur() == '\t') &&
                  self.offset < self.string.len() {
                ind.push(self.cur());
                self.right();
                mov -= 1;
            }

            for _ in 0..mov {
                self.right();
            }

            ind
        } else {
            String::new()
        };

        window.set_title(&format!("{}{}{}","self (", &self.url, ") Changed"));
        self.string = self.string[0 .. self.offset].to_string() +
            &c.to_string() +
            &self.string[self.offset .. self.string.len()];

        self.right();

        if c == '\n' {
            for c in ind.chars() {
                self.insert(c, window);
            }

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

    fn save(&mut self, window: &Window) {
        match self.file {
            Some(ref mut file) => {
                file.seek(SeekFrom::Start(0));
                file.write(&self.string.as_bytes());
                file.sync();
            }
            None => {
                let mut save_window = {
                    const width: usize = 400;
                    const height: usize = 200;
                    Window::new((window.x() + (window.width()/2 - width/2) as isize),
                                (window.y() + (window.height()/2 - height/2) as isize),
                                width,
                                height,
                                "Save As").unwrap()
                };
                if let Some(event) = save_window.poll() {
                    //TODO: Create a Save/Cancel button for file saving
                    // and prompt the user for asking to save
                }
            }
        }
    }

    fn get_x(&self) -> usize {
        let mut x = 0;
        for (n, c) in self.string.chars().enumerate() {
            if c == '\n' {
                x = 0;
            } else {
                x += 1;
            }
            if n >= self.offset {
                break;
            }
        }
        x
    }

    fn draw_content(&mut self, window: &mut Window) {
        let mut redraw = false;

        {
			let GRAY = Color::rgba(128, 128, 128, 128);			
            window.set(Color::WHITE);

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
                        window.rect(8 * col, 16 * row, 8, 16, GRAY);
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
                        window.char(8 * col, 16 * row, c, Color::BLACK);
                    }
                    col += 1;
                }

                offset += 1;
            }

            if offset == self.offset {
                if col >= 0 && col < cols && row >= 0 && row < rows {
                        window.rect(8 * col, 16 * row, 8, 16, GRAY);
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
                                     &("Editor (".to_string() + url + ")")).unwrap();

        self.url = url.to_string();
        self.file = File::open(&self.url);

        self.reload();
        self.draw_content(&mut window);

        let mut mode = Mode::Normal;

        let mut last_change = String::new();
        let mut multiplier: Option<u32> = None;
        let mut swap = 0;
        let mut period = String::new();
        let mut is_recording = false;
        let mut clipboard = String::new();

        while let Option::Some(event) = window.poll() {
            match event.to_option() {
                EventOption::Key(key_event) => {
                    if key_event.pressed {
                        cmd::exec(self, &mut mode, &mut multiplier, &mut last_change, key_event, &mut window, &mut swap, &mut period, &mut is_recording, &mut clipboard);
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
        Some(arg) => Editor::new().main(&arg),
        None => Editor::new().main("none://"),
    }
}
