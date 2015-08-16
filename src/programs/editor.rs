use core::clone::Clone;

use graphics::color::*;
use graphics::size::*;
use graphics::window::*;

use programs::common::*;

pub struct Editor {
    window: Window,
    string: String,
    loading: bool,
    offset: usize,
    scroll: Point
}

impl SessionItem for Editor {
    fn new() -> Editor {
        Editor {
            window: Window{
                point: Point::new(420, 300),
                size: Size::new(576, 400),
                title: "Editor".to_string(),
                title_color: Color::new(0, 0, 0),
                border_color: Color::new(255, 255, 255),
                content_color: Color::alpha(0, 0, 0, 196),
                shaded: false,
                closed: false,
                dragging: false,
                last_mouse_point: Point::new(0, 0),
                last_mouse_event: MouseEvent {
                    x: 0,
                    y: 0,
                    left_button: false,
                    right_button: false,
                    middle_button: false,
                    valid: false
                }
            },
            string: String::new(),
            loading: false,
            offset: 0,
            scroll: Point::new(0, 0)
        }
    }

    #[allow(unused_variables)]
    fn load(&mut self, url: &URL){
        self.window.title = "Editor Loading (".to_string() + url.to_string() + ")";

        self.string = String::new();
        self.offset = 0;
        self.scroll = Point::new(0, 0);
        self.loading = true;

        let self_ptr: *mut Editor = self;
        let url_copy = url.clone();
        url.open_async(box move |mut resource: Box<Resource>|{
            let editor;
            unsafe {
                editor = &mut *self_ptr;
            }

            let mut vec: Vec<u8> = Vec::new();
            match resource.read_to_end(&mut vec){
                Option::Some(len) => editor.string = String::from_utf8(&vec),
                Option::None => ()
            }

            editor.window.title = "Editor (".to_string() + url_copy.to_string() + ")";
            editor.loading = false;
        });
    }

    fn draw(&mut self, display: &Display, events: &mut Vec<URL>) -> bool{
        if ! self.window.draw(display){
            return self.loading;
        }

        if ! self.window.shaded {
            let scroll = self.scroll;
            let mut offset = 0;

            let mut col = -scroll.x;
            let cols = self.window.size.width as isize / 8;

            let mut row = -scroll.y;
            let rows = self.window.size.height as isize / 16;
            for c in self.string.chars() {
                if offset == self.offset{
                    if col >= 0 && col < cols && row >= 0 && row < rows{
                        display.char(Point::new(self.window.point.x + 8*col, self.window.point.y + 16*row), '_', Color::new(128, 128, 128));
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

                        let mut event = URL::new();
                        event.scheme = "r".to_string();
                        event.path.push(String::from_num(REDRAW_ALL));
                        events.push(event);
                    }
                }

                if c == '\n' {
                    col = -scroll.x;
                    row += 1;
                }else if c == '\t' {
                    col += 8 - col % 8;
                }else{
                    if col >= 0 && col < cols && row >= 0 && row < rows{
                        let point = Point::new(self.window.point.x + 8 * col, self.window.point.y + 16 * row);
                        display.char(point, c, Color::new(255, 255, 255));
                    }
                    col += 1;
                }

                offset += 1;
            }

            if offset == self.offset {
                if col >= 0 && col < cols && row >= 0 && row < rows{
                    display.char(Point::new(self.window.point.x + 8 * col, self.window.point.y + 16 * row), '_', Color::new(128, 128, 128));
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

                    let mut event = URL::new();
                    event.scheme = "r".to_string();
                    event.path.push(String::from_num(REDRAW_ALL));
                    events.push(event);
                }
            }
        }

        return true;
    }

    #[allow(unused_variables)]
    fn on_key(&mut self, events: &mut Vec<URL>, key_event: KeyEvent){
        if key_event.pressed {
            match key_event.scancode {
                0x01 => self.window.closed = true,
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
                    self.string = self.string.substr(0, self.offset) + self.string.substr(self.offset + 1, self.string.len() - self.offset - 1);
                },
                _ => ()
            }

            match key_event.character {
                '\x00' => (),
                '\x08' => if self.offset > 0 {
                    self.string = self.string.substr(0, self.offset - 1) + self.string.substr(self.offset, self.string.len() - self.offset);
                    self.offset -= 1;
                },
                '\x1B' => (),
                _ => {
                    self.string = self.string.substr(0, self.offset) + key_event.character + self.string.substr(self.offset, self.string.len() - self.offset);
                    self.offset += 1;
                }
            }
        }
    }

    #[allow(unused_variables)]
    fn on_mouse(&mut self, events: &mut Vec<URL>, mouse_point: Point, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return self.window.on_mouse(mouse_point, mouse_event, allow_catch);
    }
}
