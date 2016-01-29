use std::collections::VecDeque;
use std::mem::size_of;
use std::{ptr, slice};

use super::{Color, Display, Event, Image, ImageRoi};

use system::error::{Error, Result, EINVAL};

pub struct Window {
    pub x: i32,
    pub y: i32,
    image: Image,
    events: VecDeque<Event>,
}

impl Window {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Window {
        Window {
            x: x,
            y: y,
            image: Image::new(w, h),
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

    pub fn draw(&mut self, display: &mut Display) {
        let mut display_roi = display.roi(self.x, self.y, self.width(), self.height());
        display_roi.blend(&self.as_roi());
    }

    pub fn event(&mut self, event: Event) {
        self.events.push_back(event);
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result {
        if buf.len() == size_of::<Event>() {
            if let Some(event) = self.events.pop_front() {
                unsafe { ptr::write(buf.as_mut_ptr() as *mut Event, event) };
                Ok(size_of::<Event>())
            } else {
                Ok(0)
            }
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
