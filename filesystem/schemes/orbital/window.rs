use std::collections::VecDeque;
use std::ops::DerefMut;

use orbital::{Color, Point, Size, Event, KeyEvent, MouseEvent, QuitEvent};

use super::display::Display;

/// A window
pub struct Window {
    /// The position of the window
    pub point: Point,
    /// The size of the window
    pub size: Size,
    /// The title of the window
    pub title: String,
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
    dragging: bool,
    last_mouse_event: MouseEvent,
    events: VecDeque<Event>,
    ptr: *mut Window,
}

impl Window {
    /// Create a new window
    pub fn new(point: Point, size: Size, title: String) -> Box<Self> {
        let mut ret = box Window {
            point: point,
            size: size,
            title: title,
            content: Display::new(size.width as usize, size.height as usize),
            title_color: Color::rgb(255, 255, 255),
            border_color: Color::rgba(64, 64, 64, 128),
            focused: false,
            minimized: false,
            dragging: false,
            last_mouse_event: MouseEvent {
                x: 0,
                y: 0,
                left_button: false,
                right_button: false,
                middle_button: false,
            },
            events: VecDeque::new(),
            ptr: 0 as *mut Window,
        };

        unsafe {
            ret.ptr = ret.deref_mut();

            if ret.ptr as usize > 0 {
                (*super::session_ptr).add_window(ret.ptr);
            }
        }

        ret
    }

    /// Poll the window
    pub fn poll(&mut self) -> Option<Event> {
        self.events.pop_front()
    }

    /// Redraw the window
    pub fn redraw(&mut self) {
        self.content.flip();
        unsafe {
            (*super::session_ptr).redraw = true;
            (*super::session_ptr).redraw();
        }
    }

    /// Draw the window using a `Display`
    pub fn draw(&mut self, display: &Display, font: usize) {
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
            for c in self.title.chars() {
                if cursor.x + 8 <= self.point.x + self.size.width as i32 {
                    display.char(cursor, c, self.title_color, font);
                }
                cursor.x += 8;
            }

            cursor.x = self.point.x + self.size.width as i32 - 8;
            display.char(cursor, 'X', self.title_color, font);

            display.rect(Point::new(self.point.x - 2, self.point.y),
                         Size::new(2, self.size.height),
                         self.border_color);
            display.rect(Point::new(self.point.x - 2, self.point.y + self.size.height as i32),
                         Size::new(self.size.width + 4, 2),
                         self.border_color);
            display.rect(Point::new(self.point.x + self.size.width as i32, self.point.y),
                         Size::new(2, self.size.height),
                         self.border_color);

            unsafe {
                display.image(self.point,
                          self.content.onscreen as *const Color,
                          Size::new(self.content.width as u32, self.content.height as u32));
            }
        }
    }

    /// Called on key press
    pub fn on_key(&mut self, key_event: KeyEvent) {
        self.events.push_back(key_event.to_event());
    }

    fn on_window_decoration(&self, x: isize, y: isize) -> bool {
        !self.minimized && x >= -2 && x < self.size.width as isize + 4 && y >= -18 && y < 0
    }

    fn on_window_body(&self, x: isize, y: isize) -> bool {
        !self.minimized && x >= 0 && x < self.size.width as isize && y >= 0 &&
        y < self.size.height as isize
    }

    fn mouse_button_pressed(mouse_event: &MouseEvent) -> bool {
        mouse_event.left_button || mouse_event.middle_button || mouse_event.right_button
    }

    /// Called on mouse movement
    pub fn on_mouse(&mut self,
                    orig_mouse_event: MouseEvent,
                    allow_catch: bool,
                    active_window: bool)
                    -> bool {
        let mut mouse_event = orig_mouse_event;

        mouse_event.x -= self.point.x as i32;
        mouse_event.y -= self.point.y as i32;

        let mut caught = false;

        if allow_catch &&
           (active_window ||
            (Window::mouse_button_pressed(&mouse_event) &&
             !Window::mouse_button_pressed(&self.last_mouse_event))) {
            if self.on_window_body(mouse_event.x as isize, mouse_event.y as isize) {
                caught = true;
            } else if self.on_window_decoration(mouse_event.x as isize, mouse_event.y as isize) {
                caught = true;

                if mouse_event.left_button {
                    if !self.last_mouse_event.left_button {
                        self.dragging = true;
                    }

                    if mouse_event.x >= self.size.width as i32 - 8 {
                        self.events.push_back(QuitEvent.to_event());
                    }
                } else {
                    self.dragging = false;
                }

                if mouse_event.right_button {
                    if !self.last_mouse_event.right_button {
                        self.minimized = !self.minimized;
                    }
                }

                if mouse_event.middle_button {
                    self.events.push_back(QuitEvent.to_event());
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
            self.events.push_back(mouse_event.to_event());
        }

        caught
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            if self.ptr as usize > 0 {
                (*super::session_ptr).remove_window(self.ptr);
            }
        }
    }
}
