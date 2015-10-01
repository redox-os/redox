use alloc::boxed::*;

use core::mem::size_of;
use core::ptr;

use common::event::*;
use common::random::*;
use common::string::*;
use common::vec::*;

use graphics::color::*;
use graphics::point::*;
use graphics::size::*;
use graphics::window::*;

use syscall::call::*;

static mut window: *mut Box<ConsoleWindow> = 0 as *mut Box<ConsoleWindow>;

pub fn console_window<'a>() -> &'a mut Box<ConsoleWindow> {
    unsafe {
        if window as usize == 0 {
            window = sys_alloc(size_of::<Box<ConsoleWindow>>()) as *mut Box<ConsoleWindow>;
            ptr::write(window, ConsoleWindow::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(640, 480), "Console".to_string()));
            (*window).redraw();
        }
        &mut *window
    }
}

pub unsafe fn console_init() {
    window = 0 as *mut Box<ConsoleWindow>;
}

pub unsafe fn console_destroy() {
    if window as usize > 0 {
        drop(ptr::read(window));
        sys_unalloc(window as usize);
        window = 0 as *mut Box<ConsoleWindow>;
    }
}

pub fn console_title(title: &String){
    console_window().window.title = title.clone();
    console_window().redraw();
}

#[macro_export]
macro_rules! print_color {
    ($text:expr, $color:expr) => ({
        console_window().print(&$text, $color);
        console_window().redraw();
    });
}

#[macro_export]
macro_rules! print {
    ($text:expr) => (print_color!($text, Color::new(224, 224, 224)));
}

#[macro_export]
macro_rules! println {
    ($text:expr) => ({
        print!($text);
        print!(&"\n".to_string());
    });
}

#[macro_export]
macro_rules! readln {
    () => (console_window().read());
}

pub struct ConsoleChar {
    character: char,
    color: Color
}

pub struct ConsoleWindow {
    pub window: Box<Window>,
    pub output: Vec<ConsoleChar>,
    pub command: String,
    pub offset: usize,
    pub scroll: Point,
    pub wrap: bool
}

impl ConsoleWindow {
    pub fn new(point: Point, size: Size, title: String) -> Box<ConsoleWindow> {
        box ConsoleWindow {
            window: Window::new(point, size, title),
            output: Vec::new(),
            command: String::new(),
            offset: 0,
            scroll: Point::new(0, 0),
            wrap: true
        }
    }

    pub fn poll(&mut self) -> EventOption {
        self.window.poll()
    }

    pub fn print(&mut self, string: &String, color: Color){
        for c in string.chars() {
            self.output.push(ConsoleChar{ character: c, color: color });
        }
    }

    pub fn read(&mut self) -> Option<String> {
        loop {
            match self.poll() {
                EventOption::Key(key_event) => {
                    if key_event.pressed {
                        match key_event.scancode {
                            K_BKSP => if self.offset > 0 {
                                self.command = self.command.substr(0, self.offset - 1) + self.command.substr(self.offset, self.command.len() - self.offset);
                                self.offset -= 1;
                            },
                            K_DEL => if self.offset < self.command.len() {
                                self.command = self.command.substr(0, self.offset) + self.command.substr(self.offset + 1, self.command.len() - self.offset - 1);
                            },
                            K_HOME => self.offset = 0,
                            K_UP => {
                                //self.command = self.last_command.clone();
                                //self.offset = self.command.len();
                            },
                            K_LEFT => if self.offset > 0 {
                                self.offset -= 1;
                            },
                            K_RIGHT => if self.offset < self.command.len() {
                                self.offset += 1;
                            },
                            K_END => self.offset = self.command.len(),
                            K_DOWN => {
                                //self.command = String::new();
                                //self.offset = self.command.len();
                            },
                            _ => match key_event.character {
                                '\x00' => (),
                                '\n' => {
                                    let command = self.command.clone();
                                    self.command = String::new();
                                    self.offset = 0;
                                    return Option::Some(command);
                                },
                                '\x1B' => return Option::None,
                                _ => {
                                    self.command = self.command.substr(0, self.offset) + key_event.character + self.command.substr(self.offset, self.command.len() - self.offset);
                                    self.offset += 1;
                                }
                            }
                        }
                    }
                    self.redraw();
                },
                EventOption::None => sys_yield(),
                _ => ()
            }
        }
    }

    pub fn redraw(&mut self){
        let scroll = self.scroll;

        let mut col = -scroll.x;
        let cols = self.window.content.width as isize / 8;
        let mut row = -scroll.y;
        let rows = self.window.content.height as isize / 16;

        {
            let content = &self.window.content;
            content.set(Color::new(0, 0, 0));

            for c in self.output.iter(){
                if self.wrap && col >= cols {
                    col = -scroll.x;
                    row += 1;
                }

                if c.character == '\n' {
                    col = -scroll.x;
                    row += 1;
                }else if c.character == '\t' {
                    col += 8 - col % 8;
                }else{
                    if col >= 0 && col < cols && row >= 0 && row < rows{
                        content.char(Point::new(8 * col, 16 * row), c.character, c.color);
                    }
                    col += 1;
                }
            }

            if col > -scroll.x {
                col = -scroll.x;
                row += 1;
            }

            if col >= 0 && col < cols && row >= 0 && row < rows{
                content.char(Point::new(8 * col, 16 * row), '#', Color::new(255, 255, 255));
                col += 2;
            }

            let mut i = 0;
            for c in self.command.chars(){
                if self.wrap && col >= cols {
                    col = -scroll.x;
                    row += 1;
                }

                if self.offset == i && col >= 0 && col < cols && row >= 0 && row < rows{
                    content.char(Point::new(8 * col, 16 * row), '_', Color::new(255, 255, 255));
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

                i += 1;
            }

            if self.wrap && col >= cols {
                col = -scroll.x;
                row += 1;
            }

            if self.offset == i && col >= 0 && col < cols && row >= 0 && row < rows{
                content.char(Point::new(8 * col, 16 * row), '_', Color::new(255, 255, 255));
            }
        }

        self.window.redraw();

        if row >= rows {
            self.scroll.y += row - rows + 1;

            self.redraw();
        }
    }
}
