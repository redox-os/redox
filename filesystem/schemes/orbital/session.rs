// TODO Calm down on those `as` integer converts (especially the lossy ones).

use std::fs::File;
use std::io::Read;

use orbital::{BmpFile, Color, Point, Size, Event, EventOption, KeyEvent, MouseEvent};

use super::display::Display;
use super::window::Window;

/// A session
pub struct Session {
    /// The display
    pub display: Box<Display>,
    /// The font
    pub font: Vec<u8>,
    /// The cursor icon
    pub cursor: BmpFile,
    /// The background image
    pub background: BmpFile,
    /// The last mouse event
    pub last_mouse_event: MouseEvent,
    /// Open windows
    pub windows: Vec<*mut Window>,
    /// Ordered windows
    pub windows_ordered: Vec<*mut Window>,
    /// Redraw
    pub redraw: bool,
}

impl Session {
    /// Create new session
    pub fn new() -> Option<Box<Self>> {
        if let Some(display) = unsafe { Display::root() } {
            let mut ret = box Session {
                display: display,
                font: Vec::new(),
                cursor: BmpFile::default(),
                background: BmpFile::default(),
                last_mouse_event: MouseEvent {
                    x: 0,
                    y: 0,
                    left_button: false,
                    middle_button: false,
                    right_button: false,
                },
                windows: Vec::new(),
                windows_ordered: Vec::new(),
                redraw: true,
            };

            match File::open("file:/ui/unifont.font") {
                Ok(mut file) => {
                    let mut vec = Vec::new();
                    file.read_to_end(&mut vec);
                    ret.font = vec;
                }
                Err(err) => println!("Failed to open font: {}", err),
            }

            ret.cursor = BmpFile::from_path("file:/ui/cursor.bmp");
            if !ret.cursor.has_data() {
                println!("Failed to read cursor");
            }

            ret.background = BmpFile::from_path("file:/ui/background.bmp");
            if !ret.background.has_data() {
                println!("Failed to read background");
            }

            Some(ret)
        } else {
            None
        }
    }

    pub unsafe fn add_window(&mut self, add_window_ptr: *mut Window) {
        self.windows.push(add_window_ptr);
        self.windows_ordered.push(add_window_ptr);
        self.redraw = true;
    }

    /// Remove a window
    pub unsafe fn remove_window(&mut self, remove_window_ptr: *mut Window) {
        let mut i = 0;
        while i < self.windows.len() {
            let mut remove = false;

            match self.windows.get(i) {
                Some(window_ptr) => {
                    if *window_ptr == remove_window_ptr {
                        remove = true;
                    } else {
                        i += 1;
                    }
                }
                None => break,
            }

            if remove {
                self.windows.remove(i);
            }
        }

        i = 0;
        while i < self.windows_ordered.len() {
            let mut remove = false;

            match self.windows_ordered.get(i) {
                Some(window_ptr) => {
                    if *window_ptr == remove_window_ptr {
                        remove = true;
                    } else {
                        i += 1;
                    }
                }
                None => break,
            }

            if remove {
                self.windows_ordered.remove(i);
            }
        }

        self.redraw = true;
    }

    fn on_key(&mut self, key_event: KeyEvent) {
        if !self.windows.is_empty() {
            match self.windows.get(self.windows.len() - 1) {
                Some(window_ptr) => unsafe {
                    (**window_ptr).on_key(key_event);
                    self.redraw = true;
                },
                None => (),
            }
        }
    }

    fn on_mouse(&mut self, mouse_event: MouseEvent) {
        let mut catcher = -1;

        if mouse_event.y >= self.display.height as i32 - 32 {
            if !mouse_event.left_button && self.last_mouse_event.left_button {
                let mut x = 0;

                let mut chars = 32;
                while chars > 4 &&
                      (x as usize + (chars * 8 + 3 * 4) * self.windows.len()) > self.display.width {
                    chars -= 1;
                }

                x += 4;
                for window_ptr in self.windows_ordered.iter() {
                    let w = (chars * 8 + 2 * 4) as usize;
                    if mouse_event.x >= x && mouse_event.x < x + w as i32 {
                        for j in 0..self.windows.len() {
                            match self.windows.get(j) {
                                Some(catcher_window_ptr) => {
                                    if catcher_window_ptr == window_ptr {
                                        unsafe {
                                            if j == self.windows.len() - 1 {
                                                (**window_ptr).minimized = !(**window_ptr)
                                                                                .minimized;
                                            } else {
                                                catcher = j as isize;
                                                (**window_ptr).minimized = false;
                                            }
                                        }
                                        break;
                                    }
                                }
                                None => break,
                            }
                        }
                        self.redraw = true;
                        break;
                    }
                    x += w as i32;
                }
            }
        } else {
            let mut active_window = true;
            for reverse_i in 0..self.windows.len() {
                let i = self.windows.len() - 1 - reverse_i;
                match self.windows.get(i) {
                    Some(window_ptr) => unsafe {
                        if (**window_ptr).on_mouse(mouse_event, catcher < 0, active_window) {
                            catcher = i as isize;

                            self.redraw = true;
                            break;
                        }
                    },
                    None => (),
                }
                active_window = false;
            }
        }

        if catcher >= 0 && catcher < self.windows.len() as isize - 1 {
            let window_ptr = self.windows.remove(catcher as usize);
            self.windows.push(window_ptr);
        }

        if mouse_event.x != self.last_mouse_event.x || mouse_event.y != self.last_mouse_event.y {
            self.redraw = true;
        }

        self.last_mouse_event = mouse_event;
    }

    /// Redraw screen
    pub unsafe fn redraw(&mut self) {
        if self.redraw {
            let mouse_point = Point::new(self.last_mouse_event.x, self.last_mouse_event.y);
            self.display.set(Color::rgb(75, 163, 253));
            if self.background.has_data() {
                self.display
                    .image(Point::new((self.display.width as i32 - self.background.width() as i32) /
                                      2,
                                      (self.display.height as i32 -
                                       self.background.height() as i32) /
                                      2),
                           (&self.background).as_ptr(),
                           Size::new(self.background.width() as u32,
                                     self.background.height() as u32));
            }

            for i in 0..self.windows.len() {
                match self.windows.get(i) {
                    Some(window_ptr) => {
                        (**window_ptr).focused = i == self.windows.len() - 1;
                        (**window_ptr).draw(&self.display, self.font.as_ptr() as usize);
                    }
                    None => (),
                }
            }

            self.display.rect(Point::new(0, self.display.height as i32 - 32),
                              Size::new(self.display.width as u32, 32),
                              Color::rgba(0, 0, 0, 128));

            let mut x = 0;

            let mut chars = 32;
            while chars > 4 &&
                  (x as usize + (chars * 8 + 3 * 4) * self.windows.len()) > self.display.width {
                chars -= 1;
            }

            x += 4;
            for window_ptr in self.windows_ordered.iter() {
                let w = (chars * 8 + 2 * 4) as usize;
                self.display.rect(Point::new(x, self.display.height as i32 - 32),
                                  Size::new(w as u32, 32),
                                  (**window_ptr).border_color);
                x += 4;

                let mut i = 0;
                for c in (**window_ptr).title.chars() {
                    if c != '\0' {
                        self.display.char(Point::new(x, self.display.height as i32 - 24),
                                          c,
                                          (**window_ptr).title_color,
                                          self.font.as_ptr() as usize);
                    }
                    x += 8;
                    i += 1;
                    if i >= chars {
                        break;
                    }
                }
                while i < chars {
                    x += 8;
                    i += 1;
                }
                x += 8;
            }

            if self.cursor.has_data() {
                self.display.image_alpha(mouse_point,
                                         (&self.cursor).as_ptr(),
                                         Size::new(self.cursor.width() as u32,
                                                   self.cursor.height() as u32));
            } else {
                self.display.char(Point::new(mouse_point.x - 3, mouse_point.y - 9),
                                  'X',
                                  Color::rgb(255, 255, 255),
                                  self.font.as_ptr() as usize);
            }

            self.display.flip();

            self.redraw = false;
        }
    }

    pub fn event(&mut self, event: &Event) {
        match event.to_option() {
            EventOption::Mouse(mouse_event) => self.on_mouse(mouse_event),
            EventOption::Key(key_event) => self.on_key(key_event),
            _ => (),
        }
    }
}
