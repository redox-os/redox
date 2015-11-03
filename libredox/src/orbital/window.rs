use alloc::boxed::Box;

use string::{String, ToString};

use core::ops::DerefMut;

use event::*;

use graphics::color::Color;
use graphics::display::Display;
use graphics::point::Point;
use graphics::size::Size;
use orbital::session::Session;
use vec::Vec;
use fs::File;
use io::*;
use core::{cmp,mem};

/// A window
pub struct Window {
    /// The position of the window
    pub point: Point,
    /// The size of the window
    pub size: Size,
    /// The title of the window
    pub t: String,
    /// The content of the window
    pub content: Box<Display>,
    /// The color of the window title
    pub title_color: Color,
    /// The color of the border
    pub border_color: Color,
    /// Is the window focused?
    pub focused: bool,
    /// Is the window minimized?
    pub minimized: bool,
    font: Vec<u8>,
    dragging: bool,
    last_mouse_event: MouseEvent,
    events: Vec<Event>,
    ptr: *mut Window,
}

impl Window {
    /// Create a new window
    pub fn new(x: isize, y: isize, w: usize, h: usize, title: &str) -> Option<Box<Self>> {
        let point = Point::new(x, y);
        let size = Size { width: w, height: h };
        let mut font = Vec::new();
        if let Some(mut font_file) = File::open("file:///ui/unifont.font") {
            font_file.read_to_end(&mut font);
        }
        let mut ret = box Window {
            point: point,
            size: size,
            t: title.to_string(),
            content: Display::new(size.width, size.height),
            title_color: Color::rgb(255, 255, 255),
            border_color: Color::rgba(64, 64, 64, 128),
            focused: false,
            minimized: false,
            font: font,
            dragging: false,
            last_mouse_event: MouseEvent {
                x: 0,
                y: 0,
                left_button: false,
                right_button: false,
                middle_button: false,
            },
            events: Vec::new(),
            ptr: 0 as *mut Window,
        };

        unsafe {
            ret.ptr = ret.deref_mut();
            if ret.ptr as usize > 0 {
                (*Session::session()).add_window(ret.ptr);
            }
        }

        Some(ret)
    }

    /* functions from old version */
    /// Draw a pixel
    pub fn pixel(&mut self, x: isize, y: isize, color: Color) {
        self.content.pixel(Point::new(x,y), color);
    }

    /// Draw a character, using the loaded font
    pub fn char(&mut self, x: isize, y: isize, c: char, color: Color) {
        let cursor = Point::new(x, y);
        self.content.char(&self.font, cursor, c, color)
    }

    /// Set entire window to a color
    pub fn set(&mut self, color: Color) {
        self.content.rect(Point::new(0,0), self.size, color);
    }

    pub fn rect(&mut self, start_x: isize, start_y: isize, w: usize, h: usize, color: Color) {
        for y in start_y..start_y + h as isize {
            for x in start_x..start_x + w as isize {
                self.pixel(x, y, color);
            }
        }
    }

    /// Poll the window (new)
    pub fn poll(&mut self) -> Option<Event> {
        loop {
            if !self.events.is_empty() {
                return Some(self.events.remove(0));
            } else {
                return None
            }
        }
    }

    pub fn image(&mut self, x: isize, y: isize, w: usize, h: usize, data: &[Color]) {
        let point = Point::new(x,y);
        let size = Size::new(w,h);
        // TODO: make the types happy and remove the nested loop
        // LazyOxen
        /*
        unsafe {
            self.content.image(point,
                               data as *const u32, 
                               size);
        }
        */
        let mut i = 0;
        let Point{ x: start_x, y: start_y } = point;
        let w = cmp::min(start_x as usize + size.width, self.size.width);
        let h = cmp::min(start_y as usize + size.height, self.size.height);
        let len = data.len();
        for y in start_y..start_y + h as isize {
            for x in start_x..start_x + w as isize {
               if i < len {
                   self.pixel(x, y, data[i])
               }
               i += 1;
            }
        }
    }

    pub fn sync(&mut self) -> bool {
        self.redraw();
        true
    }

    pub fn x(&self) -> isize {
        self.point.x
    }

    pub fn y(&self) -> isize {
        self.point.y
    }

    pub fn width(&self) -> usize {
        self.size.width
    }

    pub fn height(&self) -> usize {
        self.size.height
    }

    pub fn title(&self) -> String {
        self.t.clone()
    }

    pub fn set_title(&mut self, title: &str) {
        self.t = title.to_string();
    }
    /* end of the old functions */

    /// Redraw the window
    pub fn redraw(&mut self) {
        self.content.flip();
        //TODO: fix this
        // LazyOxen
        unsafe {
            (*Session::session()).redraw = true;
        }
    }

    /// Draw the window using a `Display`
    pub fn draw(&mut self, display: &Display) {
        if self.focused {
            self.border_color = Color::rgba(128, 128, 128, 192);
        } else {
            self.border_color = Color::rgba(64, 64, 64, 128);
        }

        if self.minimized {
            self.title_color = Color::rgb(0, 0, 0);
        } else {
            self.title_color = Color::rgb(255, 255, 255);

            display.rect(Point::new(self.point.x - 2, self.point.y - 18),
                         Size::new(self.size.width + 4, 18),
                         self.border_color);

            let mut cursor = Point::new(self.point.x, self.point.y - 17);
            for c in self.t.chars() {
                if cursor.x + 8 <= self.point.x + self.size.width as isize {
                    display.char(&self.font, cursor, c, self.title_color);
                }
                cursor.x += 8;
            }

            display.rect(Point::new(self.point.x - 2, self.point.y),
                         Size::new(2, self.size.height),
                         self.border_color);
            display.rect(Point::new(self.point.x - 2,
                                    self.point.y + self.size.height as isize),
                         Size::new(self.size.width + 4, 2),
                         self.border_color);
            display.rect(Point::new(self.point.x + self.size.width as isize,
                                    self.point.y),
                         Size::new(2, self.size.height),
                         self.border_color);

            unsafe {
                display.image(self.point,
                              self.content.onscreen as *const u32,
                              Size::new(self.content.width, self.content.height));
            }
        }
    }

    /// Called on key press
    pub fn on_key(&mut self, key_event: KeyEvent) {
        self.events.push(key_event.to_event());
    }

    fn on_window_decoration(&self, x: isize, y: isize) -> bool {
        !self.minimized && x >= -2 &&
            x < self.size.width as isize + 4 &&
            y >= -18 &&
            y < 0
    }

    fn on_window_body(&self, x: isize, y: isize) -> bool {
        !self.minimized && x >= 0 &&
            x < self.size.width as isize &&
            y >= 0 &&
            y < self.size.height as isize
    }

    /// Called on mouse movement
    pub fn on_mouse(&mut self, orig_mouse_event: MouseEvent, allow_catch: bool) -> bool {
        let mut mouse_event = orig_mouse_event;

        mouse_event.x -= self.point.x;
        mouse_event.y -= self.point.y;

        let mut caught = false;

        if allow_catch {
            if mouse_event.left_button {
                if self.on_window_body(mouse_event.x, mouse_event.y) {
                    caught = true;
                }else if self.on_window_decoration(mouse_event.x, mouse_event.y) {
                    caught = true;
                    if !self.last_mouse_event.left_button {
                        self.dragging = true;
                    }
                }
            } else {
                self.dragging = false;
            }

            if mouse_event.right_button {
                if self.on_window_body(mouse_event.x, mouse_event.y) {
                    caught = true;
                }else if self.on_window_decoration(mouse_event.x, mouse_event.y) {
                    caught = true;
                    if !self.last_mouse_event.right_button {
                        self.minimized = !self.minimized;
                    }
                }
            }

            if mouse_event.middle_button {
                if self.on_window_body(mouse_event.x, mouse_event.y) {
                    caught = true;
                }else if self.on_window_decoration(mouse_event.x, mouse_event.y) {
                    caught = true;
                }
            }

            if self.dragging {
                self.point.x += orig_mouse_event.x - self.last_mouse_event.x;
                self.point.y += orig_mouse_event.y - self.last_mouse_event.y;
                caught = true;
            }
        } else {
            self.dragging = false;
        }

        self.last_mouse_event = orig_mouse_event;

        if caught && !self.dragging {
            self.events.push(mouse_event.to_event());
        }

        caught
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            if self.ptr as usize > 0 {
                (*Session::session()).remove_window(self.ptr);
            }
        }
    }
}
/// Event iterator
pub struct EventIter<'a> {
    window: &'a mut Window,
}

impl<'a> Iterator for EventIter<'a> {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        self.window.poll()
    }
}
