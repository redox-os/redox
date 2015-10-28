use super::package::*;
use super::executor::*;

use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;

use common::event::{Event, EventOption, KeyEvent, MouseEvent};
use common::scheduler;

use graphics::bmp::BMPFile;
use graphics::color::Color;
use graphics::display::Display;
use graphics::point::Point;
use graphics::size::Size;
use graphics::window::Window;

use schemes::KScheme;
use schemes::{Resource, URL, VecResource};

/// A session
pub struct Session {
    /// The display
    pub display: Box<Display>,
    /// The background image
    pub background: BMPFile,
    /// The cursor icon
    pub cursor: BMPFile,
    /// The mouse point
    pub mouse_point: Point,
    last_mouse_event: MouseEvent,
    /// The scheme items
    pub items: Vec<Box<KScheme>>, // FIXME: Vec<Box<T>> is equiv to Vec<T>
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
        unsafe {
            box Session {
                display: Display::root(),
                background: BMPFile::new(),
                cursor: BMPFile::new(),
                mouse_point: Point::new(0, 0),
                last_mouse_event: MouseEvent {
                    x: 0,
                    y: 0,
                    left_button: false,
                    middle_button: false,
                    right_button: false,
                },
                items: Vec::new(),
                packages: Vec::new(),
                windows: Vec::new(),
                windows_ordered: Vec::new(),
                redraw: true,
            }
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
                Some(window_ptr) => if *window_ptr == remove_window_ptr {
                    remove = true;
                } else {
                    i += 1;
                },
                None => break,
            }

            if remove{
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

    pub unsafe fn on_irq(&mut self, irq: u8) {
        let reenable = scheduler::start_no_ints();
        for mut item in self.items.iter_mut() {
            item.on_irq(irq);
        }
        scheduler::end_no_ints(reenable);
    }

    pub unsafe fn on_poll(&mut self) {
        let reenable = scheduler::start_no_ints();
        for mut item in self.items.iter_mut() {
            item.on_poll();
        }
        scheduler::end_no_ints(reenable);
    }

    /// Open a new resource
    pub fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
        if url.scheme().len() == 0 {
            let mut list = String::new();

            for item in self.items.iter() {
                let scheme = item.scheme();
                if scheme.len() > 0 {
                    if list.len() > 0 {
                        list = list + "\n" + scheme;
                    } else {
                        list = scheme.to_string();
                    }
                }
            }

            Some(box VecResource::new(URL::new(), list.into_bytes()))
        } else {
            for mut item in self.items.iter_mut() {
                if item.scheme() == url.scheme() {
                    return item.open(url);
                }
            }
            None
        }
    }

    fn on_key(&mut self, key_event: KeyEvent) {
        if self.windows.len() > 0 {
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
                    if package.icon.data.len() > 0 {
                        if mouse_event.x >= x &&
                           mouse_event.x < x + package.icon.size.width as isize {
                            execute(&package.binary, &package.url, Vec::new());
                        }
                        x += package.icon.size.width as isize;
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
                    let w = (chars*8 + 2*4) as usize;
                    if mouse_event.x >= x && mouse_event.x < x + w as isize {
                        for j in 0..self.windows.len() {
                            match self.windows.get(j) {
                                Some(catcher_window_ptr) =>
                                    if catcher_window_ptr == window_ptr {
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

        self.last_mouse_event = mouse_event;
    }

    /// Redraw screen
    pub unsafe fn redraw(&mut self) {
        if self.redraw {
            self.display.set(Color::new(75, 163, 253));
            if self.background.data.len() > 0 {
                self.background.draw(&self.display,
                                     Point::new((self.display.width as isize -
                                                 self.background.size.width as isize) /
                                                2,
                                                (self.display.height as isize -
                                                 self.background.size.height as isize) /
                                                2));
            }

            for i in 0..self.windows.len() {
                match self.windows.get(i) {
                    Some(window_ptr) => {
                        (**window_ptr).focused = i == self.windows.len() - 1;
                        (**window_ptr).draw(&self.display);
                    }
                    None => (),
                }
            }

            self.display.rect(Point::new(0, self.display.height as isize - 32),
                              Size::new(self.display.width, 32),
                              Color::alpha(0, 0, 0, 128));

            let mut x = 0;
            for package in self.packages.iter() {
                if package.icon.data.len() > 0 {
                    let y = self.display.height as isize - package.icon.size.height as isize;
                    if self.mouse_point.y >= y && self.mouse_point.x >= x &&
                       self.mouse_point.x < x + package.icon.size.width as isize {
                        self.display.rect(Point::new(x, y),
                                          package.icon.size,
                                          Color::alpha(128, 128, 128, 128));

                       self.display.rect(Point::new(x, y - 16),
                                         Size::new(package.name.len() * 8, 16),
                                         Color::alpha(0, 0, 0, 128));

                        let mut c_x = x;
                        for c in package.name.chars() {
                            self.display
                                .char(Point::new(c_x, y - 16), c, Color::new(255, 255, 255));
                            c_x += 8;
                        }
                    }
                    package.icon.draw(&self.display, Point::new(x, y));
                    x += package.icon.size.width as isize;
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
                let w = (chars*8 + 2*4) as usize;
                self.display.rect(Point::new(x, self.display.height as isize - 32),
                                  Size::new(w, 32),
                                  (**window_ptr).border_color);
                x += 4;

                let mut i = 0;
                for c in (**window_ptr).title.chars() {
                    if c != '\0' {
                        self.display.char(Point::new(x, self.display.height as isize - 24),
                                          c,
                                          (**window_ptr).title_color);
                    }
                    x += 8;
                    i += 1;
                    if i >= chars {
                        break;
                    }
                }
                x += 8;
            }

            if self.cursor.data.len() > 0 {
                self.display.image_alpha(self.mouse_point,
                                         self.cursor.data.as_ptr(),
                                         self.cursor.size);
            } else {
                self.display.char(Point::new(self.mouse_point.x - 3, self.mouse_point.y - 9),
                                  'X',
                                  Color::new(255, 255, 255));
            }

            let reenable = scheduler::start_no_ints();

            self.display.flip();

            self.redraw = false;

            scheduler::end_no_ints(reenable);
        }
    }

    pub fn event(&mut self, event: Event) {
        match event.to_option() {
            EventOption::Mouse(mouse_event) => self.on_mouse(mouse_event),
            EventOption::Key(key_event) => self.on_key(key_event),
            _ => (),
        }
    }
}
