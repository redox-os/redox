use core::result::Result;

use common::debug::*;
use common::string::*;
use common::vector::*;

use drivers::disk::*;
use drivers::keyboard::*;
use drivers::mouse::*;

use filesystems::unfs::*;

use graphics::color::*;
use graphics::point::*;
use graphics::size::*;
use graphics::window::*;

use programs::editor::*;
use programs::executor::*;
use programs::program::*;
use programs::viewer::*;

pub struct FileManager {
    window: Window,
    files: Vector<String>,
    selected: isize
}

impl FileManager {
    pub unsafe fn new() -> FileManager {
        let mut size = Size::new(0, 0);

        let files = UnFS::new(Disk::new()).list();

        if size.height < files.len() * 16 {
            size.height = files.len() * 16;
        }

        for file in files.as_slice() {
            if size.width < (file.len() + 1) * 8 {
                size.width = (file.len() + 1) * 8 ;
            }
        }

        FileManager {
            window: Window{
                point: Point::new(10, 50),
                size: size,
                title: String::from_str("File Manager"),
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
            files: files,
            selected: -1
        }
    }
}

impl Program for FileManager {
    unsafe fn draw(&self, session: &mut Session) -> bool{
        let display = &session.display;

        if ! self.window.draw(display) {
            return false;
        }

        if ! self.window.shaded {
            let mut i = 0;
            let mut row = 0;
            for string in self.files.as_slice() {
                let mut col = 0;
                for c_ptr in string.as_slice() {
                    let c = *c_ptr;
                    if c == '\n' {
                        col = 0;
                        row += 1;
                    }else if c == '\t' {
                        col += 8 - col % 8;
                    }else{
                        let color;
                        if i == self.selected {
                            color = Color::new(128, 128, 128);
                        }else{
                            color = Color::new(255, 255, 255);
                        }

                        if col < self.window.size.width / 8 && row < self.window.size.height / 16 {
                            let point = Point::new(self.window.point.x + 8*col as isize, self.window.point.y + 16*row as isize);
                            display.char(point, c, color);
                            col += 1;
                        }
                    }
                    if col >= self.window.size.width / 8 {
                        col = 0;
                        row += 1;
                    }
                }
                row += 1;
                i += 1;
            }
        }

        return true;
    }

    #[allow(unused_variables)]
    unsafe fn on_key(&mut self, session: &mut Session, key_event: KeyEvent){
        if key_event.pressed {
            match key_event.scancode {
                0x01 => self.selected = -1,
                0x1C => if self.selected >= 0 && self.selected < self.files.len() as isize {
                            match self.files.get(self.selected as usize) {
                                Result::Ok(file) => {
                                    d("Loading ");
                                    file.d();
                                    dl();
                                    if file.ends_with(&String::from_str(".md")){
                                        session.add_program(box Editor::new(file));
                                    }else if file.ends_with(&String::from_str(".bin")){
                                        d("Load executable ");
                                        file.d();
                                        dl();

                                        session.add_program(box Executor::new(file));
                                    }else if file.ends_with(&String::from_str(".bmp")){
                                        session.add_program(box Viewer::new(file));
                                    }else{
                                        d("No program found!\n");
                                    }
                                },
                                Result::Err(_) => ()
                            }
                        },
                0x47 => self.selected = 0,
                0x48 => if self.selected > 0 {
                            self.selected -= 1;
                        },
                0x4F => self.selected = self.files.len() as isize - 1,
                0x50 => if self.selected < self.files.len() as isize - 1 {
                            self.selected += 1;
                        },
                _ => ()
            }
        }
    }

    unsafe fn on_mouse(&mut self, session: &mut Session, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        let mouse_point = session.mouse_point;
        if self.window.on_mouse(mouse_point, mouse_event, allow_catch) {
            if ! self.window.shaded {
                let mut i = 0;
                let mut row = 0;
                for string in self.files.as_slice() {
                    let mut col = 0;
                    for c_ptr in string.as_slice() {
                        let c = *c_ptr;
                        if c == '\n' {
                            col = 0;
                            row += 1;
                        }else if c == '\t' {
                            col += 8 - col % 8;
                        }else{
                            if col < self.window.size.width / 8 && row < self.window.size.height / 16 {
                                let point = Point::new(self.window.point.x + 8*col as isize, self.window.point.y + 16*row as isize);
                                if mouse_point.x >= point.x && mouse_point.x < point.x + 8 && mouse_point.y >= point.y && mouse_point.y < point.y + 16 {
                                    self.selected = i;
                                }
                                col += 1;
                            }
                        }
                        if col >= self.window.size.width / 8 {
                            col = 0;
                            row += 1;
                        }
                    }
                    row += 1;
                    i += 1;
                }
            }

            return true;
        }else{
            return false;
        }
    }
}