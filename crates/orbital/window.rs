use std::cmp::max;
use std::collections::VecDeque;
use std::mem::size_of;
use std::{ptr, slice};

use super::{Color, Display, Event, Font, Image, ImageRoi};

use system::error::{Error, Result, EINVAL};

pub struct Window {
    pub x: i32,
    pub y: i32,
    image: Image,
    title: String,
    events: VecDeque<Event>,
}

impl Window {
    pub fn new(x: i32, y: i32, w: i32, h: i32, title: String) -> Window {
        Window {
            x: x,
            y: y,
            image: Image::new(w, h),
            title: title,
            events: VecDeque::new()
        }
    }

    pub fn width(&self) -> i32 {
        self.image.width()
    }

    pub fn height(&self) -> i32 {
        self.image.height()
    }

    pub fn as_roi(&mut self) -> ImageRoi {
        self.image.as_roi()
    }

    pub fn roi(&mut self, x: i32, y: i32, w: i32, h: i32) -> ImageRoi {
        self.image.roi(x, y, w, h)
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.width() && y < self.y + self.height()
    }

    pub fn title_contains(&self, x: i32, y: i32) -> bool {
        ! self.title.is_empty() && x >= self.x && y >= self.y - 18 && x < self.x + self.width() && y < self.y
    }

    pub fn exit_contains(&self, x: i32, y: i32) -> bool {
        ! self.title.is_empty() && x >= max(self.x, self.x + self.width() - 10)  && y >= self.y - 18 && x < self.x + self.width() && y < self.y
    }

    pub fn draw(&mut self, display: &mut Display, focused: bool) {
        if ! self.title.is_empty() {
            if focused {
                display.roi(self.x, self.y - 18, self.width(), 18).set(Color::rgba(192, 192, 192, 224));
            } else {
                display.roi(self.x, self.y - 18, self.width(), 18).set(Color::rgba(64, 64, 64, 224));
            }

            let mut x = self.x + 2;
            for c in self.title.chars() {
                if x + 8 <= self.x + self.width() - 10 {
                    display.roi(x, self.y - 17, 8, 16).blend(&Font::render(c, Color::rgb(255, 255, 255)).as_roi());
                } else {
                    break;
                }
                x += 8;
            }

            x = max(self.x + 2, self.x + self.width() - 10);
            if x + 10 <= self.x + self.width() {
                display.roi(x, self.y - 17, 8, 16).blend(&Font::render('X', Color::rgb(255, 255, 255)).as_roi());
            }
        }
        let mut display_roi = display.roi(self.x, self.y, self.width(), self.height());
        display_roi.blend(&self.as_roi());
    }

    pub fn event(&mut self, event: Event) {
        self.events.push_back(event);
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.len() >= size_of::<Event>() {
            let mut i = 0;
            while i <= buf.len() - size_of::<Event>() {
                if let Some(event) = self.events.pop_front() {
                    unsafe { ptr::write(buf.as_mut_ptr().offset(i as isize) as *mut Event, event) };
                    i += size_of::<Event>();
                } else {
                    break;
                }
            }
            Ok(i)
        } else {
            Err(Error::new(EINVAL))
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let old = self.image.data_mut();
        let new = unsafe { slice::from_raw_parts(buf.as_ptr() as *const Color, buf.len() / size_of::<Color>()) };

        let mut i = 0;
        while i < old.len() && i < new.len() {
            old[i] = new[i];
            i += 1;
        }

        Ok(i * size_of::<Color>())
    }

    pub fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;
        let path_str = format!("orbital:/{}/{}/{}/{}/{}", self.x, self.y, self.width(), self.height(), self.title);
        let path = path_str.as_bytes();
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }
        Ok(i)
    }
}
