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

use programs::program::*;

pub struct Editor {
    window: Window,
    filename: String,
    string: String,
    offset: usize
}

impl Editor {
    pub unsafe fn new(file: &String) -> Editor {
        let mut ret = Editor {
            window: Window{
                point: Point::new(420, 300),
                size: Size::new(576, 400),
                title: String::from_str("Editor"),
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
            offset: 0
        };

        if file.len() > 0{
            d("Load text file ");
            file.d();
            dl();

            ret.load(file);
        }

        return ret;
    }

    unsafe fn clear(&mut self){
        self.window.title = String::from_str("Editor");
        self.filename = String::new();
        self.string = String::new();
        self.offset = 0;
    }

    unsafe fn load(&mut self, filename: &String){
        self.clear();
        let unfs = UnFS::new(Disk::new());
        let dest = unfs.load(filename);
        if dest > 0 {
            self.filename = filename.clone();
            self.window.title = String::from_str("Editor (") + filename + String::from_str(")");
            self.string = String::from_c_str(dest as *const u8);
            self.offset = self.string.len();
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
        unfs.save(&self.filename, data);
        unalloc(data);
        d("Saved\n");
    }
}

impl Program for Editor {
    unsafe fn draw(&self, session: &mut Session) -> bool{
        let display = &session.display;

        if ! self.window.draw(display){
            return false;
        }

        if ! self.window.shaded {
            let mut offset = 0;
            let mut row = 0;
            let mut col = 0;
            for c_ptr in self.string.as_slice() {
                if offset == self.offset && col < self.window.size.width / 8 && row < self.window.size.height / 16 {
                    display.char(Point::new(self.window.point.x + 8*col as isize, self.window.point.y + 16*row as isize), '_', Color::new(128, 128, 128));
                }

                let c = *c_ptr;
                if c == '\n' {
                    col = 0;
                    row += 1;
                }else if c == '\t' {
                    col += 8 - col % 8;
                }else{
                    if col < self.window.size.width / 8 && row < self.window.size.height / 16 {
                        let point = Point::new(self.window.point.x + 8*col as isize, self.window.point.y + 16*row as isize);
                        display.char(point, c, Color::new(255, 255, 255));
                        col += 1;
                    }
                }
                if col >= self.window.size.width / 8 {
                    col = 0;
                    row += 1;
                }

                offset += 1;
            }

            if offset == self.offset && col < self.window.size.width / 8 && row < self.window.size.height / 16 {
                display.char(Point::new(self.window.point.x + 8*col as isize, self.window.point.y + 16*row as isize), '_', Color::new(128, 128, 128));
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
                    match self.string.get(self.offset - i) {
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
                    match self.string.get(i) {
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