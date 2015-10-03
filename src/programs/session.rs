use super::package::*;
use super::executor::*;

use alloc::boxed::Box;

use core::{cmp, ptr, mem};

use common::event::{self, Event, EventOption, KeyEvent, MouseEvent};
use common::resource::{NoneResource, Resource, ResourceType, URL, VecResource};
use common::scheduler::*;
use common::string::{String, ToString};
use common::vec::Vec;

use graphics::bmp::*;
use graphics::color::Color;
use graphics::display::Display;
use graphics::point::Point;
use graphics::size::Size;
use graphics::window::Window;

use programs::common::SessionItem;

pub struct Session {
    pub display: Display,
    pub background: BMP,
    pub cursor: BMP,
    pub mouse_point: Point,
    last_mouse_event: MouseEvent,
    pub items: Vec<Box<SessionItem>>,
    pub packages: Vec<Box<Package>>,
    pub windows: Vec<*mut Window>,
    pub windows_ordered: Vec<*mut Window>,
    pub redraw: usize,
}

impl Session {
    pub fn new() -> Session {
        unsafe {
            Session {
                display: Display::root(),
                background: BMP::new(),
                cursor: BMP::new(),
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
                redraw: event::REDRAW_ALL,
            }
        }
    }

    pub unsafe fn add_window(&mut self, add_window_ptr: *mut Window) {
        self.windows.push(add_window_ptr);
        self.windows_ordered.push(add_window_ptr);
        self.redraw = cmp::max(self.redraw, event::REDRAW_ALL);
    }

    pub unsafe fn remove_window(&mut self, remove_window_ptr: *mut Window) {
        let mut i = 0;
        while i < self.windows.len() {
            let mut remove = false;

            match self.windows.get(i) {
                Option::Some(window_ptr) => if *window_ptr == remove_window_ptr {
                    remove = true;
                } else {
                    i += 1;
                },
                Option::None => break,
            }

            if remove {
                self.windows.remove(i);
            }
        }

        i = 0;
        while i < self.windows_ordered.len() {
            let mut remove = false;

            match self.windows_ordered.get(i) {
                Option::Some(window_ptr) => if *window_ptr == remove_window_ptr {
                    remove = true;
                } else {
                    i += 1;
                },
                Option::None => break,
            }

            if remove {
                self.windows_ordered.remove(i);
            }
        }

        self.redraw = cmp::max(self.redraw, event::REDRAW_ALL);
    }

    pub unsafe fn on_irq(&mut self, irq: u8) {
        for item in self.items.iter() {
            let reenable = start_no_ints();
            item.on_irq(irq);
            end_no_ints(reenable);
        }
    }

    pub unsafe fn on_poll(&mut self) {
        for item in self.items.iter() {
            let reenable = start_no_ints();
            item.on_poll();
            end_no_ints(reenable);
        }
    }

    pub fn open(&self, url: &URL) -> Box<Resource> {
        if url.scheme().len() == 0 {
            let mut list = String::new();

            for item in self.items.iter() {
                let scheme = item.scheme();
                if scheme.len() > 0 {
                    if list.len() > 0 {
                        list = list + "\n" + scheme;
                    } else {
                        list = scheme;
                    }
                }
            }

            box VecResource::new(URL::new(), ResourceType::Dir, list.to_utf8())
        } else {
            for item in self.items.iter() {
                if item.scheme() == url.scheme() {
                    return item.open(url);
                }
            }
            box NoneResource
        }
    }

    fn on_key(&mut self, key_event: KeyEvent) {
        if self.windows.len() > 0 {
            match self.windows.get(self.windows.len() - 1) {
                Option::Some(window_ptr) => {
                    unsafe {
                        (**window_ptr).on_key(key_event);
                        self.redraw = cmp::max(self.redraw, event::REDRAW_ALL);
                    }
                }
                Option::None => (),
            }
        }
    }

    fn on_mouse(&mut self, mouse_event: MouseEvent) {
        let mut catcher = -1;

        if mouse_event.y >= self.display.height as isize - 32 {
            if mouse_event.left_button && !self.last_mouse_event.left_button {
                let mut x = 0;
                for package in self.packages.iter() {
                    if package.icon.data.len() > 0 {
                        if mouse_event.x >= x &&
                           mouse_event.x < x + package.icon.size.width as isize {
                            execute(&package.binary, &package.url, &Vec::new());
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
                                Option::Some(catcher_window_ptr) =>
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
                                Option::None => break,
                            }
                        }
                        self.redraw = cmp::max(self.redraw, event::REDRAW_ALL);
                        break;
                    }
                    x += w as isize;
                }
            }
        } else {
            for reverse_i in 0..self.windows.len() {
                let i = self.windows.len() - 1 - reverse_i;
                match self.windows.get(i) {
                    Option::Some(window_ptr) => unsafe {
                        if reverse_i == 0 ||
                           (mouse_event.left_button && !self.last_mouse_event.left_button) {
                            if (**window_ptr).on_mouse(mouse_event, catcher < 0) {
                                catcher = i as isize;

                                self.redraw = cmp::max(self.redraw, event::REDRAW_ALL);
                            }
                        }
                    },
                    Option::None => (),
                }
            }
        }

        if catcher >= 0 && catcher < self.windows.len() as isize - 1 {
            match self.windows.remove(catcher as usize) {
                Option::Some(window_ptr) => self.windows.push(window_ptr),
                Option::None => (),
            }
        }

        self.last_mouse_event = mouse_event;
    }

    pub unsafe fn redraw(&mut self) {
        if self.redraw > event::REDRAW_NONE {
            //if self.redraw >= REDRAW_ALL {
            self.display.set(Color::new(64, 64, 64));
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
                    Option::Some(window_ptr) => {
                        (**window_ptr).focused = i == self.windows.len() - 1;
                        (**window_ptr).draw(&self.display);
                    }
                    Option::None => (),
                }
            }

            self.display.rect(Point::new(0, self.display.height as isize - 32),
                              Size::new(self.display.width, 32),
                              Color::new(0, 0, 0));

            let mut x = 0;
            for package in self.packages.iter() {
                if package.icon.data.len() > 0 {
                    let y = self.display.height as isize - package.icon.size.height as isize;
                    if self.mouse_point.y >= y && self.mouse_point.x >= x &&
                       self.mouse_point.x < x + package.icon.size.width as isize {
                        self.display.rect(Point::new(x, y),
                                          package.icon.size,
                                          Color::new(128, 128, 128));

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

                for i in 0..chars {
                    let c = (**window_ptr).title[i];
                    if c != '\0' {
                        self.display.char(Point::new(x, self.display.height as isize - 24),
                                          c,
                                          (**window_ptr).title_color);
                    }
                    x += 8;
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
            //}

            let reenable = start_no_ints();

            self.display.flip();

            /*
            if self.cursor.data.len() > 0 {
                self.display.image_alpha_onscreen(self.mouse_point, self.cursor.data.as_ptr(), self.cursor.size);
            } else {
                self.display.char_onscreen(Point::new(self.mouse_point.x - 3, self.mouse_point.y - 9), 'X', Color::new(255, 255, 255));
            }
            */

            self.redraw = event::REDRAW_NONE;

            end_no_ints(reenable);
        }
    }

    pub fn event(&mut self, event: Event) {
        match event.to_option() {
            EventOption::Mouse(mouse_event) => self.on_mouse(mouse_event),
            EventOption::Key(key_event) => self.on_key(key_event),
            EventOption::Redraw(redraw_event) =>
                self.redraw = cmp::max(self.redraw, redraw_event.redraw),
            EventOption::Open(open_event) => {
                let url_string = open_event.url_string;

                if url_string.ends_with(".bin".to_string()) {
                    execute(&URL::from_string(&url_string),
                            &URL::new(),
                            &Vec::new());
                } else {
                    for package in self.packages.iter() {
                        let mut accepted = false;
                        for accept in package.accepts.iter() {
                            if url_string.ends_with(accept.substr(1, accept.len() - 1)) {
                                accepted = true;
                                break;
                            }
                        }
                        if accepted {
                            let mut args: Vec<String> = Vec::new();
                            args.push(url_string.clone());
                            execute(&package.binary, &package.url, &args);
                            break;
                        }
                    }
                }
            }
            _ => (),
        }
    }
}
