use alloc::boxed::*;

use core::mem::size_of;
use core::ptr;

use string::*;
use vec::Vec;

use event::*;

use orbital::*;

use rand_old::*;

static mut window: *mut ConsoleWindow = 0 as *mut ConsoleWindow;

/// Create a new console window
pub fn console_window() -> &'static mut ConsoleWindow {
    unsafe {
        if window as usize == 0 {
            window = Box::into_raw(ConsoleWindow::new((rand() % 400 + 50) as isize,
                                          (rand() % 300 + 50) as isize,
                                          640,
                                          480,
                                          "Console"));
            (*window).sync();
        }
        &mut *window
    }
}

/// Initialize console
pub unsafe fn console_init() {
    window = 0 as *mut ConsoleWindow;
}

/// Destroy the console
pub unsafe fn console_destroy() {
    if window as usize > 0 {
        drop(Box::from_raw(window));
        window = 0 as *mut ConsoleWindow;
    }
}

/// Set the title of the console window
// TODO: Move this to a `Window` trait?
pub fn console_title(title: &str) {
    console_window().set_title(title);
}

/// Print to console
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        console_window().print(&format!($($arg)*), [224, 224, 224, 255]);
        console_window().sync();
    });
}

/// Print to console with color
#[macro_export]
macro_rules! print_color {
    ($color:expr, $($arg:tt)*) => ({
        console_window().print(&format!($($arg)*), $color);
        console_window().sync();
    });
}

/// Print new line to console
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        print!($($arg)*);
        print!("\n");
    });
}

/// Print new line to console with color
#[macro_export]
macro_rules! println_color {
    ($color:expr, $($arg:tt)*) => ({
        print_color!($color, $($arg)*);
        print_color!($color, "\n");
    });
}

/// Read a line from the console
#[macro_export]
macro_rules! readln {
    () => (console_window().read());
}

/// A console char
pub struct ConsoleChar {
    /// The char
    character: char,
    /// The color
    color: [u8; 4],
}

/// A console window
pub struct ConsoleWindow {
    /// The window
    pub window: Box<Window>,
    /// The char buffer
    pub output: Vec<ConsoleChar>,
    /// The current input command
    pub command: String,
    /// Offset
    pub offset: usize,
    /// Scroll distance x
    pub scroll_x: isize,
    /// Scroll distance y
    pub scroll_y: isize,
    /// Wrap the text, if true
    pub wrap: bool,
}

impl ConsoleWindow {
    /// Create a new console window
    pub fn new(x: isize, y: isize, w: usize, h: usize, title: &str) -> Box<Self> {
        box ConsoleWindow {
            window: Window::new(x, y, w, h, title).unwrap(),
            output: Vec::new(),
            command: String::new(),
            offset: 0,
            scroll_x: 0,
            scroll_y: 0,
            wrap: true,
        }
    }

    /// Set the window title
    pub fn set_title(&mut self, title: &str) {
        //TODO THIS IS A HACK, should use self.window.setTitle(title);
        self.window = Window::new(self.window.x(),
                                  self.window.y(),
                                  self.window.width(),
                                  self.window.height(),
                                  title).unwrap();
    }

    /// Poll the window
    pub fn poll(&mut self) -> Option<Event> {
        self.window.poll()
    }

    /// Print to the window
    pub fn print(&mut self, string: &str, color: [u8; 4]) {
        for c in string.chars() {
            self.output.push(ConsoleChar {
                character: c,
                color: color,
            });
        }
    }

    /// Read input
    pub fn read(&mut self) -> Option<String> {
        while let Some(event) = self.poll() {
            match event.to_option() {
                EventOption::Key(key_event) => {
                    if key_event.pressed {
                        match key_event.scancode {
                            K_BKSP => if self.offset > 0 {
                                self.command = self.command[0 .. self.offset - 1].to_string() +
                                               &self.command[self.offset .. self.command.len()];
                                self.offset -= 1;
                            },
                            K_DEL => if self.offset < self.command.len() {
                                self.command =
                                    self.command[0 .. self.offset].to_string() +
                                    &self.command[self.offset + 1 .. self.command.len() - 1];
                            },
                            K_HOME => self.offset = 0,
                            K_UP => {
                                //self.command = self.last_command.clone();
                                //self.offset = self.command.len();
                            }
                            K_LEFT => if self.offset > 0 {
                                self.offset -= 1;
                            },
                            K_RIGHT => if self.offset < self.command.len() {
                                self.offset += 1;
                            },
                            K_END => self.offset = self.command.len(),
                            K_DOWN => {
                                //self.command.clear()
                                //self.offset = self.command.len();
                            }
                            _ => match key_event.character {
                                '\x00' => (),
                                '\n' => {
                                    let command = self.command.clone();
                                    self.command.clear();
                                    self.offset = 0;
                                    return Some(command);
                                }
                                '\x1B' => break,
                                _ => {
                                    self.command = self.command[0 .. self.offset].to_string() +
                                                   &key_event.character.to_string() +
                                                   &self.command[self.offset .. self.command.len()];
                                    self.offset += 1;
                                }
                            },
                        }
                    }
                    self.sync();
                }
                _ => (),
            }
        }

        return None;
    }

    /// Redraw the window
    pub fn sync(&mut self) {
        let scroll_x = self.scroll_x;
        let scroll_y = self.scroll_y;

        let mut col = -scroll_x;
        let cols = self.window.width() as isize / 8;
        let mut row = -scroll_y;
        let rows = self.window.height() as isize / 16;

        {
            self.window.set([0, 0, 0, 255]);

            for c in self.output.iter() {
                if self.wrap && col >= cols {
                    col = -scroll_x;
                    row += 1;
                }

                if c.character == '\n' {
                    col = -scroll_x;
                    row += 1;
                } else if c.character == '\t' {
                    col += 8 - col % 8;
                } else {
                    if col >= 0 && col < cols && row >= 0 && row < rows {
                        self.window.char(8 * col, 16 * row, c.character, c.color);
                    }
                    col += 1;
                }
            }

            if col > -scroll_x {
                col = -scroll_x;
                row += 1;
            }

            if col >= 0 && col < cols && row >= 0 && row < rows {
                self.window.char(8 * col, 16 * row, '#', [255, 255, 255, 255]);
                col += 2;
            }

            let mut i = 0;
            for c in self.command.chars() {
                if self.wrap && col >= cols {
                    col = -scroll_x;
                    row += 1;
                }

                if self.offset == i && col >= 0 && col < cols && row >= 0 && row < rows {
                    self.window.char(8 * col, 16 * row, '_', [255, 255, 255, 255]);
                }

                if c == '\n' {
                    col = -scroll_x;
                    row += 1;
                } else if c == '\t' {
                    col += 8 - col % 8;
                } else {
                    if col >= 0 && col < cols && row >= 0 && row < rows {
                        self.window.char(8 * col, 16 * row, c, [255, 255, 255, 255]);
                    }
                    col += 1;
                }

                i += 1;
            }

            if self.wrap && col >= cols {
                col = -scroll_x;
                row += 1;
            }

            if self.offset == i && col >= 0 && col < cols && row >= 0 && row < rows {
                self.window.char(8 * col, 16 * row, '_', [255, 255, 255, 255]);
            }
        }

        self.window.sync();

        if row >= rows {
            self.scroll_y += row - rows + 1;

            self.sync();
        }
    }
}
