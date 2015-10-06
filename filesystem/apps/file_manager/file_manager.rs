use redox::*;

pub struct FileManager {
    folder_icon: BMP,
    file_icon: BMP,
    files: Vec<String>,
    selected: isize,
}

impl FileManager {
    pub fn new() -> Self {
        let file_icon;
        {
            let mut resource = File::open("file:///ui/mimetypes/unknown.bmp");

            let mut vec: Vec<u8> = Vec::new();
            resource.read_to_end(&mut vec);

            file_icon = BMP::from_data(&vec);
        }

        let folder_icon;
        {
            let mut resource = File::open("file:///ui/places/folder.bmp");

            let mut vec: Vec<u8> = Vec::new();
            resource.read_to_end(&mut vec);

            folder_icon = BMP::from_data(&vec);
        }

        return FileManager {
            file_icon: file_icon,
            folder_icon: folder_icon,
            files: Vec::new(),
            selected: -1,
        };
    }

    fn draw_content(&mut self, window: &mut Window) {
        window.set([255, 255, 255, 255]);

        let mut i = 0;
        let mut row = 0;
        for string in self.files.iter() {
            if i == self.selected {
                let width = window.width();
                window.rect(0, 32 * row as isize, width, 32, [224, 224, 224, 255]);
            }

            if string.ends_with('/') {
                window.image(0,
                             32 * row as isize,
                             self.folder_icon.width(),
                             self.folder_icon.height(),
                             self.folder_icon.as_slice());
            } else {
                window.image(0,
                             32 * row as isize,
                             self.file_icon.width(),
                             self.file_icon.height(),
                             self.file_icon.as_slice());
            }

            let mut col = 0;
            for c in string.chars() {
                if c == '\n' {
                    col = 0;
                    row += 1;
                } else if c == '\t' {
                    col += 8 - col % 8;
                } else {
                    if col < window.width() / 8 && row < window.height() / 32 {
                        window.char(8 * col as isize + 40,
                                    32 * row as isize + 8,
                                    c,
                                    [0, 0, 0, 255]);
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

    fn main(&mut self, path: &str) {
        let mut width = 160;
        let mut height = 0;
        {
            let mut resource = File::open(path);

            let mut vec: Vec<u8> = Vec::new();
            resource.read_to_end(&mut vec);

            for file in unsafe { String::from_utf8_unchecked(vec) }.split('\n') {
                if width < 40 + (file.len() + 1) * 8 {
                    width = 40 + (file.len() + 1) * 8;
                }
                self.files.push(file.to_string());
            }

            if height < self.files.len() * 32 {
                height = self.files.len() * 32;
            }
        }

        let mut window = Window::new((rand() % 400 + 50) as isize,
                                     (rand() % 300 + 50) as isize,
                                     width,
                                     height,
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
                                                url_string: path.to_string() + &file,
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
                            if mouse_event.y >= 32 * row as isize &&
                               mouse_event.y < 32 * row as isize + 32 {
                                self.selected = i;
                                redraw = true;
                            }

                            if c == '\n' {
                                col = 0;
                                row += 1;
                            } else if c == '\t' {
                                col += 8 - col % 8;
                            } else {
                                if col < window.width() / 8 && row < window.height() / 32 {
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
        Option::Some(arg) => FileManager::new().main(arg),
        Option::None => FileManager::new().main("file:///"),
    }
}
