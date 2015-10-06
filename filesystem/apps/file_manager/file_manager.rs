use redox::*;

pub struct FileManager {
    files: Vec<String>,
    selected: isize,
}

impl FileManager {
    pub fn new() -> Self {
        return FileManager {
            files: Vec::new(),
            selected: -1,
        };
    }

    fn draw_content(&mut self, window: &mut Window) {
        window.set([0, 0, 0, 255]);

        let mut i = 0;
        let mut row = 0;
        for string in self.files.iter() {
            let mut col = 0;
            for c in string.chars() {
                if c == '\n' {
                    col = 0;
                    row += 1;
                } else if c == '\t' {
                    col += 8 - col % 8;
                } else {
                    let color;
                    if i == self.selected {
                        color = [128, 128, 128, 255];
                    } else {
                        color = [255, 255, 255, 255];
                    }

                    if col < window.width() / 8 && row < window.height() / 16 {
                        window.char(8 * col as isize, 16 * row as isize, c, color);
                        col += 1;
                    }
                }
                if col >= window.width() / 8 {
                    col = 0;
                    row += 1;
                }
            }
            row += 1;
            i += 1;
        }

        window.sync();
    }

    fn main(&mut self, path: String) {
        let mut width = 160;
        let mut height = 0;
        {
            let mut resource = File::open(&path);

            let mut vec: Vec<u8> = Vec::new();
            resource.read_to_end(&mut vec);

            for file in unsafe { String::from_utf8_unchecked(vec) }.split('\n') {
                if width < (file.len() + 1) * 8 {
                    width = (file.len() + 1) * 8;
                }
                self.files.push(file.to_string());
            }

            if height < self.files.len() * 16 {
                height = self.files.len() * 16;
            }
        }

        let mut window = Window::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize,
                                     width, height,
                                     &("File Manager (".to_string() + &path + ")"));

        self.draw_content(&mut window);

        while let Option::Some(event) = window.poll() {
            match event.to_option() {
                EventOption::Key(key_event) => {
                    if key_event.pressed {
                        match key_event.scancode {
                            K_ESC => break,
                            K_HOME => self.selected = 0,
                            K_UP => if self.selected > 0 {
                                self.selected -= 1;
                            },
                            K_END => self.selected = self.files.len() as isize - 1,
                            K_DOWN => if self.selected < self.files.len() as isize - 1 {
                                self.selected += 1;
                            },
                            _ => match key_event.character {
                                '\0' => (),
                                '\n' => {
                                    if self.selected >= 0 &&
                                       self.selected < self.files.len() as isize {
                                        match self.files.get(self.selected as usize) {
                                            Option::Some(file) => OpenEvent {
                                                url_string: path.clone() + &file,
                                            }.trigger(),
                                            Option::None => (),
                                        }
                                    }
                                }
                                _ => {
                                    let mut i = 0;
                                    for file in self.files.iter() {
                                        if file.starts_with(key_event.character) {
                                            self.selected = i;
                                            break;
                                        }
                                        i += 1;
                                    }
                                }
                            },
                        }

                        self.draw_content(&mut window);
                    }
                }
                EventOption::Mouse(mouse_event) => {
                    let mut redraw = false;
                    let mut i = 0;
                    let mut row = 0;
                    for file in self.files.iter() {
                        let mut col = 0;
                        for c in file.chars() {
                            if c == '\n' {
                                col = 0;
                                row += 1;
                            } else if c == '\t' {
                                col += 8 - col % 8;
                            } else {
                                if col < window.width() / 8 &&
                                   row < window.height() / 16 {
                                    if mouse_event.x >= 8 * col as isize &&
                                       mouse_event.x < 8 * col as isize + 8 &&
                                       mouse_event.y >= 16 * row as isize &&
                                       mouse_event.y < 16 * row as isize + 16 {
                                        self.selected = i;
                                        redraw = true;
                                    }
                                    col += 1;
                                }
                            }
                            if col >= window.width() / 8 {
                                col = 0;
                                row += 1;
                            }
                        }
                        row += 1;
                        i += 1;
                    }

                    if redraw {
                        self.draw_content(&mut window);
                    }
                }
                EventOption::None => sys_yield(),
                _ => (),
            }
        }
    }
}

pub fn main() {
    match args().get(1) {
        Option::Some(arg) => FileManager::new().main(arg.clone()),
        Option::None => FileManager::new().main("file:///".to_string()),
    }
}
