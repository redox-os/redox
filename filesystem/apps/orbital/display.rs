use std::fs::File;
use std::io::{Result, Read, Write};
use std::mem::size_of;
use std::slice;

use super::{Color, Event, Image, ImageRoi};

pub struct Display {
    file: File,
    image: Image,
}

impl Display {
    pub fn new() -> Result<Display> {
        let file = try!(File::open("display:"));

        let path = try!(file.path()).to_string();
        let res = path.split(":").nth(1).unwrap_or("");
        let width = res.split("/").nth(0).unwrap_or("").parse::<i32>().unwrap_or(0);
        let height = res.split("/").nth(1).unwrap_or("").parse::<i32>().unwrap_or(0);

        Ok(Display {
            file: file,
            image: Image::new(width, height)
        })
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

    pub fn poll(&mut self) -> Option<Event> {
        let mut event = Event::new();
        if let Ok(count) = self.file.read(&mut event) {
            if count == size_of::<Event>() {
                return Some(event);
            }
        }
        None
    }

    pub fn flip(&mut self) {
        let data = self.image.data();
        self.file.write(unsafe { slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * size_of::<Color>()) });
    }
}
