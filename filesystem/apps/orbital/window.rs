use std::mem::size_of;
use std::slice;

use super::{Color, Display, Image, ImageRoi};

use system::error::Result;

pub struct Window {
    x: i32,
    y: i32,
    image: Image
}

impl Window {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Window {
        Window {
            x: x,
            y: y,
            image: Image::new(w, h)
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

    pub fn read(&mut self, buf: &mut [u8]) -> Result {
        Ok(0)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result {
        let old = self.image.data_mut();
        let new = unsafe { & slice::from_raw_parts(buf.as_ptr() as *const Color, buf.len() / size_of::<Color>()) };

        let mut i = 0;
        while i < old.len() && i < new.len() {
            old[i] = new[i];
            i += 1;
        }

        Ok(i * size_of::<Color>())
    }
}
