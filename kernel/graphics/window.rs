extern crate alloc;

use self::alloc::boxed::*;

use core::ops::DerefMut;

use common::event::*;
use common::queue::*;
use common::scheduler;
use common::string::*;

use super::color::*;
use super::display::*;
use super::point::*;
use super::size::*;

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
    events: Queue<Event>,
    ptr: *mut Window,
}

impl Window {
    /// Create a new window
    pub fn new(point: Point, size: Size, title: String) -> Box<Self> {
        let mut ret = box Window {
            point: point,
            size: size,
            title: title,
            content: Display::new(size.width, size.height),
            title_color: Color::new(255, 255, 255),
            border_color: Color::alpha(64, 64, 64, 128),
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
            events: Queue::new(),
            ptr: 0 as *mut Window,
        };

        unsafe {
            ret.ptr = ret.deref_mut();

            if ret.ptr as usize > 0 {
                (*::session_ptr).add_window(ret.ptr);
            }
        }

        ret
    }

    /// Poll the window (new)
    pub fn poll(&mut self) -> Option<Event> {
        let event_option;
        unsafe {
            let reenable = scheduler::start_no_ints();
            event_option = self.events.pop();
            scheduler::end_no_ints(reenable);
        }

        return event_option;
    }

    /// Redraw the window
    pub fn redraw(&mut self) {
        unsafe {
            let reenable = scheduler::start_no_ints();
            self.content.flip();
            (*::session_ptr).redraw = true;
            scheduler::end_no_ints(reenable);
        }
    }

    /// Draw the window using a `Display`
    pub fn draw(&mut self, display: &Display) {
        if self.focused {
            self.border_color = Color::alpha(128, 128, 128, 192);
        } else {
            self.border_color = Color::alpha(64, 64, 64, 128);
        }

        if self.minimized {
            self.title_color = Color::new(0, 0, 0);
        } else {
            self.title_color = Color::new(255, 255, 255);

            display.rect(Point::new(self.point.x - 2, self.point.y - 18),
                         Size::new(self.size.width + 4, 18),
                         self.border_color);

            let mut cursor = Point::new(self.point.x, self.point.y - 17);
            for c in self.title.chars() {
                if cursor.x + 8 <= self.point.x + self.size.width as isize {
                    display.char(cursor, c, self.title_color);
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
                let reenable = scheduler::start_no_ints();
                display.image(self.point,
                              self.content.onscreen as *const u32,
                              Size::new(self.content.width, self.content.height));
                scheduler::end_no_ints(reenable);
            }
        }
    }

    /// Called on key press
    pub fn on_key(&mut self, key_event: KeyEvent) {
        unsafe {
            let reenable = scheduler::start_no_ints();
            self.events.push(key_event.to_event());
            scheduler::end_no_ints(reenable);
        }
    }

    /// Called on mouse movement
    pub fn on_mouse(&mut self, orig_mouse_event: MouseEvent, allow_catch: bool) -> bool {
        let mut mouse_event = orig_mouse_event;

        mouse_event.x -= self.point.x;
        mouse_event.y -= self.point.y;

        let mut caught = false;

        if allow_catch {
            if mouse_event.left_button {
                if !self.minimized && mouse_event.x >= -2 &&
                   mouse_event.x < self.size.width as isize + 4 &&
                   mouse_event.y >= -18 &&
                   mouse_event.y < self.size.height as isize + 2 {
                    caught = true;
                }

                if !self.last_mouse_event.left_button && mouse_event.x >= -2 &&
                   mouse_event.x < self.size.width as isize + 4 &&
                   mouse_event.y >= -18 && mouse_event.y < 0 {
                    self.dragging = true;
                    caught = true;
                }
            } else {
                self.dragging = false;
            }

            if mouse_event.right_button {
                if !self.minimized && mouse_event.x >= -2 &&
                   mouse_event.x < self.size.width as isize + 4 &&
                   mouse_event.y >= -18 &&
                   mouse_event.y < self.size.height as isize + 2 {
                    caught = true;
                }

                if !self.last_mouse_event.right_button && mouse_event.x >= -2 &&
                   mouse_event.x < self.size.width as isize + 4 &&
                   mouse_event.y >= -18 && mouse_event.y < 0 {
                    self.minimized = !self.minimized;
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
            unsafe {
                let reenable = scheduler::start_no_ints();
                self.events.push(mouse_event.to_event());
                scheduler::end_no_ints(reenable);
            }
        }

        caught
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            if self.ptr as usize > 0 {
                (*::session_ptr).remove_window(self.ptr);
            }
        }
    }
}
