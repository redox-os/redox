use alloc::boxed::*;

use core::ops::DerefMut;

use common::event::*;
use common::queue::*;
use common::scheduler::*;
use common::string::*;

use graphics::color::*;
use graphics::display::*;
use graphics::point::*;
use graphics::size::*;

use syscall::call::sys_window_create;
use syscall::call::sys_window_destroy;

pub struct Window {
    pub point: Point,
    pub size: Size,
    pub title: String,
    pub content: Display,
    pub title_color: Color,
    pub border_color: Color,
    pub focused: bool,
    pub minimized: bool,
    dragging: bool,
    last_mouse_event: MouseEvent,
    events: Queue<Event>,
    ptr: *mut Window
}

impl Window {
    pub fn new(point: Point, size: Size, title: String) -> Box<Window> {
        let mut ret = box Window {
            point: point,
            size: size,
            title: title,
            content: Display::new(size.width, size.height),
            title_color: Color::new(255, 255, 255),
            border_color: Color::new(64, 64, 64),
            focused: false,
            minimized: false,
            dragging: false,
            last_mouse_event: MouseEvent {
                x: 0,
                y: 0,
                left_button: false,
                right_button: false,
                middle_button: false
            },
            events: Queue::new(),
            ptr: 0 as *mut Window
        };

        unsafe {
            ret.ptr = ret.deref_mut();

            if ret.ptr as usize > 0 {
                sys_window_create(ret.ptr);
            }
        }

        ret
    }

    pub fn poll(&mut self) -> EventOption {
        let event_option;
        unsafe {
            let reenable = start_no_ints();
            event_option = self.events.pop();
            end_no_ints(reenable);
        }

        match event_option {
            Option::Some(event) => event.to_option(),
            Option::None => EventOption::None
        }
    }

    pub fn redraw(&mut self) {
        self.content.flip();
        RedrawEvent { redraw: REDRAW_ALL }.to_event().trigger();
    }

    pub fn draw(&mut self, display: &Display) {
        if self.focused {
            self.border_color = Color::new(128, 128, 128);
        } else {
            self.border_color = Color::new(64, 64, 64);
        }

        if self.minimized {
            self.title_color = Color::new(0, 0, 0);
        } else {
            self.title_color = Color::new(255, 255, 255);

            display.rect(Point::new(self.point.x - 2, self.point.y - 18), Size::new(self.size.width + 4, 18), self.border_color);

            let mut cursor = Point::new(self.point.x, self.point.y - 17);
            for c in self.title.chars() {
                if cursor.x + 8 <= self.point.x + self.size.width as isize {
                    display.char(cursor, c, self.title_color);
                }
                cursor.x += 8;
            }

            display.rect(Point::new(self.point.x - 2, self.point.y), Size::new(2, self.size.height), self.border_color);
            display.rect(Point::new(self.point.x - 2, self.point.y + self.size.height as isize), Size::new(self.size.width + 4, 2), self.border_color);
            display.rect(Point::new(self.point.x + self.size.width as isize, self.point.y), Size::new(2, self.size.height), self.border_color);

            unsafe {
                let reenable = start_no_ints();
                display.image(self.point, self.content.onscreen as *const u32, Size::new(self.content.width, self.content.height));
                end_no_ints(reenable);
            }
        }
    }

    pub fn on_key(&mut self, key_event: KeyEvent) {
        unsafe {
            let reenable = start_no_ints();
            self.events.push(key_event.to_event());
            end_no_ints(reenable);
        }
    }

    pub fn on_mouse(&mut self, mouse_event: MouseEvent, allow_catch: bool) -> bool {
        let mut caught = false;

        if allow_catch {
            if mouse_event.left_button {
                if ! self.minimized
                    && mouse_event.x >= self.point.x - 2
                    && mouse_event.x < self.point.x + self.size.width as isize + 4
                    && mouse_event.y >= self.point.y - 18
                    && mouse_event.y < self.point.y + self.size.height as isize + 2
                {
                    caught = true;
                }

                if !self.last_mouse_event.left_button
                    && mouse_event.x >= self.point.x - 2
                    && mouse_event.x < self.point.x + self.size.width as isize + 4
                    && mouse_event.y >= self.point.y - 18
                    && mouse_event.y < self.point.y
                {
                    self.dragging = true;
                    caught = true;
                }
            } else {
                self.dragging = false;
            }

            if mouse_event.right_button {
                if ! self.minimized
                    && mouse_event.x >= self.point.x - 2
                    && mouse_event.x < self.point.x + self.size.width as isize + 4
                    && mouse_event.y >= self.point.y - 18
                    && mouse_event.y < self.point.y + self.size.height as isize + 2
                {
                    caught = true;
                }

                if !self.last_mouse_event.right_button
                    && mouse_event.x >= self.point.x - 2
                    && mouse_event.x < self.point.x + self.size.width as isize + 4
                    && mouse_event.y >= self.point.y - 18
                    && mouse_event.y < self.point.y
                {
                    self.minimized = !self.minimized;
                    caught = true;
                }
            }

            if self.dragging {
                self.point.x += mouse_event.x - self.last_mouse_event.x;
                self.point.y += mouse_event.y - self.last_mouse_event.y;
                caught = true;
            }
        } else {
            self.dragging = false;
        }

        self.last_mouse_event = mouse_event;

        if caught {
            unsafe {
                let reenable = start_no_ints();
                self.events.push(mouse_event.to_event());
                end_no_ints(reenable);
            }
        }

        caught
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            if self.ptr as usize > 0 {
                sys_window_destroy(self.ptr);
            }
        }
    }
}
