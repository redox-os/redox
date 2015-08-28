use graphics::color::*;
use graphics::size::*;
use graphics::window::*;

use programs::common::*;

pub struct Editor {
    window: Window,
    events: Queue<Event>,
    closed: AtomicBool,
    url: URL,
    string: String,
    offset: usize,
    scroll: Point
}

impl Editor {
    #[inline(never)]
    pub fn new() -> Editor {
        Editor {
            window: Window::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(576, 400), "Editor".to_string()),
            events: Queue::new(),
            closed: AtomicBool::new(false),
            url: URL::new(),
            string: String::new(),
            offset: 0,
            scroll: Point::new(0, 0)
        }
    }

    fn reload(&mut self){
        self.window.title = "Editor (".to_string() + self.url.to_string() + ")";
        self.offset = 0;
        self.scroll = Point::new(0, 0);

        let mut resource = self.url.open();
        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);
        self.string = String::from_utf8(&vec);
    }

    fn save(&mut self){
        self.window.title = "Editor (".to_string() + self.url.to_string() + ") Saved";

        let mut resource = self.url.open();
        resource.write_all(&self.string.to_utf8());
    }

    fn draw_content(&mut self){
        let mut redraw = false;

        {
            let content = &self.window.content;

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
                            self.scroll.x += col - cols;
                        }
                        if row < 0 { //Too far up
                            self.scroll.y += row;
                        }else if row >= rows{ //Too far down
                            self.scroll.y += row - rows;
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
                        self.scroll.x += cols - col;
                    }
                    if row < 0 { //Too far up
                        self.scroll.y += row;
                    }else if row >= rows{ //Too far down
                        self.scroll.y += rows - row;
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
            self.draw_content();
        }
    }
}

impl SessionItem for Editor {
    fn main(&mut self, url: URL){
        self.url = url;
        self.reload();
        self.draw_content();

        while ! self.closed.load(Ordering::SeqCst) {
            let event_option;
            unsafe{
                let enable = start_no_ints();
                event_option = self.events.pop();
                end_no_ints(enable);
            }

            match event_option {
                Option::Some(event_const) => {
                    let mut event = event_const;
                    match event.code {
                        'k' => {
                            let key_event = KeyEvent::from_event(&mut event);
                            if key_event.pressed {
                                match key_event.scancode {
                                    0x3F => self.reload(),
                                    0x40 => self.save(),
                                    0x47 => self.offset = 0,
                                    0x48 => for i in 1..self.offset {
                                        match self.string[self.offset - i] {
                                            '\0' => break,
                                            '\n' => {
                                                self.offset = self.offset - i;
                                                break;
                                            },
                                            _ => ()
                                        }
                                    },
                                    0x4B => if self.offset > 0 {
                                                self.offset -= 1;
                                            },
                                    0x4D => if self.offset < self.string.len() {
                                                self.offset += 1;
                                            },
                                    0x4F => self.offset = self.string.len(),
                                    0x50 => for i in self.offset + 1..self.string.len() {
                                        match self.string[i] {
                                            '\0' => break,
                                            '\n' => {
                                                self.offset = i;
                                                break;
                                            },
                                            _ => ()
                                        }
                                    },
                                    0x53 => if self.offset < self.string.len() {
                                        self.window.title = "Editor (".to_string() + self.url.to_string() + ") Changed";
                                        self.string = self.string.substr(0, self.offset) + self.string.substr(self.offset + 1, self.string.len() - self.offset - 1);
                                    },
                                    _ => ()
                                }

                                match key_event.character {
                                    '\x00' => (),
                                    '\x08' => if self.offset > 0 {
                                        self.window.title = "Editor (".to_string() + self.url.to_string() + ") Changed";
                                        self.string = self.string.substr(0, self.offset - 1) + self.string.substr(self.offset, self.string.len() - self.offset);
                                        self.offset -= 1;
                                    },
                                    _ => {
                                        self.window.title = "Editor (".to_string() + self.url.to_string() + ") Changed";
                                        self.string = self.string.substr(0, self.offset) + key_event.character + self.string.substr(self.offset, self.string.len() - self.offset);
                                        self.offset += 1;
                                    }
                                }

                                self.draw_content();
                            }
                        },
                        _ => ()
                    }
                },
                Option::None => sys_yield()
            }
        }
    }

    fn draw(&self, display: &Display) -> bool{
        self.window.draw(display);
        return ! self.closed.load(Ordering::SeqCst);
    }

    fn on_key(&mut self, key_event: KeyEvent){
        if key_event.pressed && key_event.scancode == 1 {
            self.closed.store(true, Ordering::SeqCst);
        }
        self.events.push(key_event.to_event());
    }

    fn on_mouse(&mut self, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return self.window.on_mouse(mouse_event, allow_catch);
    }
}
