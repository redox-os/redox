use redox::{Box, String, ToString, Vec, Url};
use redox::fs::File;
use redox::io::Read;

use orbital::{BmpFile, Color, Point, Size, Event, EventOption, KeyEvent, MouseEvent};

use super::display::Display;
use super::package::*;
use super::scheduler;
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
    /// The packages (applications)
    pub packages: Vec<Box<Package>>,
    /// Open windows
    pub windows: Vec<*mut Window>,
    /// Ordered windows
    pub windows_ordered: Vec<*mut Window>,
    /// Redraw
    pub redraw: bool,
}

impl Session {
    /// Create new session
    pub fn new() -> Box<Self> {
        let mut ret = box Session {
            display: unsafe { Display::root() },
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
            packages: Vec::new(),
            windows: Vec::new(),
            windows_ordered: Vec::new(),
            redraw: true,
        };

        if let Some(mut file) = File::open("file:/ui/unifont.font") {
            let mut vec = Vec::new();
            file.read_to_end(&mut vec);
            ret.font = vec;
        } else {
            debugln!("Failed to read font");
        }

        ret.cursor = BmpFile::from_path("file:/ui/cursor.bmp");
        if !ret.cursor.has_data() {
            debugln!("Failed to read cursor");
        }


        ret.background = BmpFile::from_path("file:/ui/background.bmp");
        if !ret.background.has_data() {
            debugln!("Failed to read background");
        }

        if let Some(mut file) = File::open("file:/apps/") {
            let mut string = String::new();
            file.read_to_string(&mut string);

            for folder in string.lines() {
                if folder.ends_with('/') {
                    ret.packages
                       .push(Package::from_url(&Url::from_string("file:/apps/".to_string() +
                                                                 &folder)));
                }
            }
        } else {
            debugln!("Failed to open apps")
        }

        ret
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
                Some(window_ptr) => if *window_ptr == remove_window_ptr {
                    remove = true;
                } else {
                    i += 1;
                },
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
                Some(window_ptr) => if *window_ptr == remove_window_ptr {
                    remove = true;
                } else {
                    i += 1;
                },
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
                Some(window_ptr) => {
                    unsafe {
                        (**window_ptr).on_key(key_event);
                        self.redraw = true;
                    }
                }
                None => (),
            }
        }
    }

    fn on_mouse(&mut self, mouse_event: MouseEvent) {
        let mut catcher = -1;

        if mouse_event.y >= self.display.height as isize - 32 {
            if !mouse_event.left_button && self.last_mouse_event.left_button {
                let mut x = 0;
                for package in self.packages.iter() {
                    if !(&package.icon).is_empty() {
                        if mouse_event.x >= x && mouse_event.x < x + package.icon.width() as isize {
                            let binary = package.binary.to_string();
                            File::exec(&binary, &[]);
                        }
                        x = x + package.icon.width() as isize;
                    }
                }

                let mut chars = 32;
                while chars > 4 &&
                      (x as usize + (chars * 8 + 3 * 4) * self.windows.len()) >
                      self.display.width + 32 {
                    chars -= 1;
                }

                x += 4;
                for window_ptr in self.windows_ordered.iter() {
                    let w = (chars * 8 + 2 * 4) as usize;
                    if mouse_event.x >= x && mouse_event.x < x + w as isize {
                        for j in 0..self.windows.len() {
                            match self.windows.get(j) {
                                Some(catcher_window_ptr) => if catcher_window_ptr == window_ptr {
                                    unsafe {
                                        if j == self.windows.len() - 1 {
                                            (**window_ptr).minimized = !(**window_ptr).minimized;
                                        } else {
                                            catcher = j as isize;
                                            (**window_ptr).minimized = false;
                                        }
                                    }
                                    break;
                                },
                                None => break,
                            }
                        }
                        self.redraw = true;
                        break;
                    }
                    x += w as isize;
                }
            }
        } else {
            for reverse_i in 0..self.windows.len() {
                let i = self.windows.len() - 1 - reverse_i;
                match self.windows.get(i) {
                    Some(window_ptr) => unsafe {
                        if (**window_ptr).on_mouse(mouse_event, catcher < 0) {
                            catcher = i as isize;

                            self.redraw = true;
                        }
                    },
                    None => (),
                }
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
                self.display.image(Point::new((self.display.width as isize -
                                               self.background.width() as isize) /
                                              2,
                                              (self.display.height as isize -
                                               self.background.height() as isize) /
                                              2),
                                   (&self.background).as_ptr(),
                                   Size::new(self.background.width(), self.background.height()));
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

            self.display.rect(Point::new(0, self.display.height as isize - 32),
                              Size::new(self.display.width, 32),
                              Color::rgba(0, 0, 0, 128));

            let mut x = 0;
            for package in self.packages.iter() {
                if !(&package.icon).is_empty() {
                    let y = self.display.height as isize - package.icon.height() as isize;
                    if mouse_point.y >= y && mouse_point.x >= x &&
                       mouse_point.x < x + package.icon.width() as isize {
                        self.display.rect(Point::new(x, y),
                                          Size::new(package.icon.width(), package.icon.height()),
                                          Color::rgba(128, 128, 128, 128));

                        self.display.rect(Point::new(x, y - 16),
                                          Size::new(package.name.len() * 8, 16),
                                          Color::rgba(0, 0, 0, 128));

                        let mut c_x = x;
                        for c in package.name.chars() {
                            self.display
                                .char(Point::new(c_x, y - 16),
                                      c,
                                      Color::rgb(255, 255, 255),
                                      self.font.as_ptr() as usize);
                            c_x += 8;
                        }
                    }

                    self.display.image_alpha(Point::new(x, y),
                                             (&package.icon).as_ptr(),
                                             Size::new(package.icon.width(),
                                                       package.icon.height()));
                    x = x + package.icon.width() as isize;
                }
            }

            let mut chars = 32;
            while chars > 4 &&
                  (x as usize + (chars * 8 + 3 * 4) * self.windows.len()) >
                  self.display.width + 32 {
                chars -= 1;
            }

            x += 4;
            for window_ptr in self.windows_ordered.iter() {
                let w = (chars * 8 + 2 * 4) as usize;
                self.display.rect(Point::new(x, self.display.height as isize - 32),
                                  Size::new(w, 32),
                                  (**window_ptr).border_color);
                x += 4;

                let mut i = 0;
                for c in (**window_ptr).title.chars() {
                    if c != '\0' {
                        self.display.char(Point::new(x, self.display.height as isize - 24),
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
                                         Size::new(self.cursor.width(), self.cursor.height()));
            } else {
                self.display.char(Point::new(mouse_point.x - 3, mouse_point.y - 9),
                                  'X',
                                  Color::rgb(255, 255, 255),
                                  self.font.as_ptr() as usize);
            }

            let reenable = scheduler::start_no_ints();

            self.display.flip();

            self.redraw = false;

            scheduler::end_no_ints(reenable);
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
