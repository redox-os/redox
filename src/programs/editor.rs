use common::debug::*;
use common::memory::*;
use common::string::*;

use drivers::disk::*;
use drivers::keyboard::*;
use drivers::mouse::*;

use filesystems::unfs::*;

use graphics::color::*;
use graphics::point::*;
use graphics::size::*;
use graphics::window::*;

use programs::session::*;

pub struct Editor {
    window: Window,
    filename: String,
    string: String,
    offset: usize,
    scroll: Point
}

impl Editor {
    pub unsafe fn new(file: String) -> Editor {
        let mut ret = Editor {
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
            filename: String::new(),
            string: String::new(),
            offset: 0,
            scroll: Point::new(0, 0)
        };

        if file.len() > 0{
            ret.load(file);
        }

        return ret;
    }

    unsafe fn clear(&mut self){
        self.window.title = String::from_str("Editor");
        self.filename = String::new();
        self.string = String::new();
        self.offset = 0;
        self.scroll = Point::new(0, 0);
    }

    unsafe fn load(&mut self, filename: String){
        self.clear();
        let unfs = UnFS::new(Disk::new());
        let dest = unfs.load(filename.clone());
        if dest > 0 {
            self.filename = filename.clone();
            self.window.title = String::from_str("Editor (") + filename + String::from_str(")");
            self.string = String::from_c_str(dest as *const u8);
            unalloc(dest);
        }else{
            d("Did not find '");
            filename.d();
            d("'\n");
        }
    }

    unsafe fn save(&self){
        let unfs = UnFS::new(Disk::new());
        let data = self.string.to_c_str() as usize;
        unfs.save(self.filename.clone(), data);
        unalloc(data);
        d("Saved\n");
    }
}

impl SessionItem for Editor {
    unsafe fn draw(&mut self, session: &mut Session) -> bool{
        let display = &session.display;

        if ! self.window.draw(display){
            return false;
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

                        session.redraw = REDRAW_ALL;
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

                    session.redraw = REDRAW_ALL;
                }
            }
        }

        return true;
    }

    #[allow(unused_variables)]
    unsafe fn on_key(&mut self, session: &mut Session, key_event: KeyEvent){
        if key_event.pressed {
            match key_event.scancode {
                0x01 => self.window.closed = true,
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

    unsafe fn on_mouse(&mut self, session: &mut Session, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return self.window.on_mouse(session.mouse_point, mouse_event, allow_catch);
    }
}