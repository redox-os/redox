use std::cmp::max;
use std::collections::VecDeque;
use std::mem::size_of;
use std::{ptr, slice};

use super::{Color, Display, Event, Image, ImageRoi};

use system::error::{Error, Result, EINVAL};

pub struct Window {
    pub x: i32,
    pub y: i32,
    image: Image,
    title: String,
    events: VecDeque<Event>,
}

impl Window {
    pub fn new(x: i32, y: i32, w: i32, h: i32, title: &str) -> Window {
        Window {
            x: x,
            y: y,
            image: Image::new(w, h),
            title: title.to_string(),
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

    pub fn draw(&mut self, display: &mut Display, focused: bool) {
        let rx = max(0, -self.x);
        let ry = max(0, -self.y);
        let rw = max(0, self.width() - rx);
        let rh = max(0, self.height() - ry);
        if ! self.title.is_empty() {
            if focused {
                display.roi(self.x, self.y - 18, rw, 18).set(Color::rgba(192, 192, 192, 224));
            } else {
                display.roi(self.x, self.y - 18, rw, 18).set(Color::rgba(64, 64, 64, 224));
            }
        }
        let mut display_roi = display.roi(self.x, self.y, self.width(), self.height());
        display_roi.blend(&self.roi(rx, ry, rw, rh));
    }

    pub fn event(&mut self, event: Event) {
        self.events.push_back(event);
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result {
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

    pub fn write(&mut self, buf: &[u8]) -> Result {
        let old = self.image.data_mut();
        let new = unsafe { slice::from_raw_parts(buf.as_ptr() as *const Color, buf.len() / size_of::<Color>()) };

        let mut i = 0;
        while i < old.len() && i < new.len() {
            old[i] = new[i];
            i += 1;
        }

        Ok(i * size_of::<Color>())
    }
}
