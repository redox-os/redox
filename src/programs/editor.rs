use programs::common::*;

pub struct Editor {
    url: URL,
    string: String,
    offset: usize,
    scroll: Point
}

impl Editor {
    #[inline(never)]
    pub fn new() -> Editor {
        Editor {
            url: URL::new(),
            string: String::new(),
            offset: 0,
            scroll: Point::new(0, 0)
        }
    }

    fn reload(&mut self, window: &mut Window){
        window.title = "Editor (".to_string() + self.url.to_string() + ")";
        self.offset = 0;
        self.scroll = Point::new(0, 0);

        let mut resource = self.url.open();

        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);

        self.string = String::from_utf8(&vec);
    }

    fn save(&mut self, window: &mut Window){
        window.title = "Editor (".to_string() + self.url.to_string() + ") Saved";

        let mut resource = self.url.open();
        resource.write(&self.string.to_utf8().as_slice());
    }

    fn draw_content(&mut self, window: &mut Window){
        let mut redraw = false;

        {
            let content = &window.content;

            content.set(Color::alpha(0, 0, 0, 196));

            let scroll = self.scroll;

            let mut offset = 0;

            let mut col = -scroll.x;
            let cols = content.width as isize / 8;

            let mut row = -scroll.y;
            let rows = content.height as isize / 16;

            for c in self.string.chars() {
                if offset == self.offset{
                    if col >= 0 && col < cols && row >= 0 && row < rows{
                        content.char(Point::new(8 * col, 16 * row), '_', Color::new(128, 128, 128));
                    }else{
                        if col < 0 { //Too far to the left
                            self.scroll.x += col;
                        }else if col >= cols{ //Too far to the right
                            self.scroll.x += cols - col + 1;
                        }
                        if row < 0 { //Too far up
                            self.scroll.y += row;
                        }else if row >= rows{ //Too far down
                            self.scroll.y += rows - row + 1;
                        }

                        redraw = true;
                    }
                }

                if c == '\n' {
                    col = -scroll.x;
                    row += 1;
                }else if c == '\t' {
                    col += 8 - col % 8;
                }else{
                    if col >= 0 && col < cols && row >= 0 && row < rows{
                        content.char(Point::new(8 * col, 16 * row), c, Color::new(255, 255, 255));
                    }
                    col += 1;
                }

                offset += 1;
            }

            if offset == self.offset {
                if col >= 0 && col < cols && row >= 0 && row < rows{
                    content.char(Point::new(8 * col, 16 * row), '_', Color::new(128, 128, 128));
                }else{
                    if col < 0 { //Too far to the left
                        self.scroll.x += col;
                    }else if col >= cols{ //Too far to the right
                        self.scroll.x += cols - col + 1;
                    }
                    if row < 0 { //Too far up
                        self.scroll.y += row;
                    }else if row >= rows{ //Too far down
                        self.scroll.y += rows - row + 1;
                    }

                    redraw = true;
                }
            }

            content.flip();

            RedrawEvent {
                redraw: REDRAW_ALL
            }.to_event().trigger();
        }

        if redraw {
            self.draw_content(window);
        }
    }
}

impl SessionItem for Editor {
    fn main(&mut self, url: URL){
        let mut window = Window::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(576, 400), "Editor (Loading)".to_string());

        self.url = url;

        self.reload(&mut window);
        self.draw_content(&mut window);

        loop {
            match window.poll() {
                EventOption::Key(key_event) => {
                    if key_event.pressed {
                        match key_event.scancode {
                            K_ESC => break,
                            K_BKSP => if self.offset > 0 {
                                window.title = "Editor (".to_string() + self.url.to_string() + ") Changed";
                                self.string = self.string.substr(0, self.offset - 1) + self.string.substr(self.offset, self.string.len() - self.offset);
                                self.offset -= 1;
                            },
                            K_DEL => if self.offset < self.string.len() {
                                window.title = "Editor (".to_string() + self.url.to_string() + ") Changed";
                                self.string = self.string.substr(0, self.offset) + self.string.substr(self.offset + 1, self.string.len() - self.offset - 1);
                            },
                            K_F5 => self.reload(&mut window),
                            K_F6 => self.save(&mut window),
                            K_HOME => self.offset = 0,
                            K_UP => {
                                let mut new_offset = 0;
                                for i in 2..self.offset {
                                    match self.string[self.offset - i] {
                                        '\0' => break,
                                        '\n' => {
                                            new_offset = self.offset - i + 1;
                                            break;
                                        },
                                        _ => ()
                                    }
                                }
                                self.offset = new_offset;
                            },
                            K_LEFT => if self.offset > 0 {
                                self.offset -= 1;
                            },
                            K_RIGHT => if self.offset < self.string.len() {
                                self.offset += 1;
                            },
                            K_END => self.offset = self.string.len(),
                            K_DOWN => {
                                let mut new_offset = self.string.len();
                                for i in self.offset..self.string.len() {
                                    match self.string[i] {
                                        '\0' => break,
                                        '\n' => {
                                            new_offset = i + 1;
                                            break;
                                        },
                                        _ => ()
                                    }
                                }
                                self.offset = new_offset;
                            },
                            _ => match key_event.character {
                                '\0' => (),
                                _ => {
                                    window.title = "Editor (".to_string() + self.url.to_string() + ") Changed";
                                    self.string = self.string.substr(0, self.offset) + key_event.character + self.string.substr(self.offset, self.string.len() - self.offset);
                                    self.offset += 1;
                                }
                            }
                        }

                        self.draw_content(&mut window);
                    }
                },
                EventOption::None => sys_yield(),
                _ => ()
            }
        }
    }
}
