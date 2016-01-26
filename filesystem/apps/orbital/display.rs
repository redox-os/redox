use std::cmp::{min, max};
use std::fs::File;
use std::io::{Result, Seek, SeekFrom, Write};
use std::mem::size_of;
use std::slice;

use orbital::Color;

use super::image::Image;

pub struct Display {
    file: File,
    pub image: Image,
}

impl Display {
    pub fn new() -> Result<Display> {
        let file = try!(File::open("display:"));

        let path = try!(file.path()).to_string();
        let res = path.split(":").nth(1).unwrap_or("");
        let width = res.split("x").nth(0).unwrap_or("").parse::<i32>().unwrap_or(0);
        let height = res.split("x").nth(1).unwrap_or("").parse::<i32>().unwrap_or(0);

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

    pub fn flip(&mut self) -> Result<()> {
        let data = self.image.data();
        try!(self.file.write(unsafe { & slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * size_of::<Color>()) }));
        Ok(())
    }
}
