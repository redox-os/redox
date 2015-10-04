use redox::*;

pub struct FileManager {
    files: Vec<String>,
    selected: isize,
}

impl FileManager {
    pub fn new() -> FileManager {
        return FileManager {
            files: Vec::new(),
            selected: -1,
        };
    }

    fn draw_content(&mut self, window: &mut Window) {
        let content = &window.content;

        content.set(Color::new(0, 0, 0));

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
                        color = Color::new(128, 128, 128);
                    } else {
                        color = Color::new(255, 255, 255);
                    }

                    if col < content.width / 8 && row < content.height / 16 {
                        content.char(Point::new(8 * col as isize, 16 * row as isize),
                                     c,
                                     color);
                        col += 1;
                    }
                }
                if col >= content.width / 8 {
                    col = 0;
                    row += 1;
                }
            }
            row += 1;
            i += 1;
        }

        content.flip();

        RedrawEvent { redraw: REDRAW_ALL }
            .to_event()
            .trigger();
    }

    fn main(&mut self, path: String) {
        let mut size = Size::new(0, 0);
        {
            let mut resource = File::open(&path);

            let mut vec: Vec<u8> = Vec::new();
            resource.read_to_end(&mut vec);

            for file in String::from_utf8(&vec).split("\n".to_string()) {
                if size.width < (file.len() + 1) * 8 {
                    size.width = (file.len() + 1) * 8;
                }
                self.files.push(file);
            }

            if size.height < self.files.len() * 16 {
                size.height = self.files.len() * 16;
            }
        }

        let mut window = Window::new(Point::new((rand() % 400 + 50) as isize,
                                                (rand() % 300 + 50) as isize),
                                     size,
                                     "File Manager (".to_string() + &path + ")");

        self.draw_content(&mut window);

        loop {
            match window.poll() {
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
                                                url_string: path.clone() + file.clone(),
                                            }
                                                                      .trigger(),
                                            Option::None => (),
                                        }
                                    }
                                }
                                _ => {
                                    let mut i = 0;
                                    for file in self.files.iter() {
                                        if file[0] == key_event.character {
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
                    if !window.minimized {
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
                                    if col < window.size.width / 8 &&
                                       row < window.size.height / 16 {
                                        let point = Point::new(window.point.x + 8 * col as isize,
                                                               window.point.y + 16 * row as isize);
                                        if mouse_event.x >= point.x &&
                                           mouse_event.x < point.x + 8 &&
                                           mouse_event.y >= point.y &&
                                           mouse_event.y < point.y + 16 {
                                            self.selected = i;
                                            redraw = true;
                                        }
                                        col += 1;
                                    }
                                }
                                if col >= window.size.width / 8 {
                                    col = 0;
                                    row += 1;
                                }
                            }
                            row += 1;
                            i += 1;
                        }
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
