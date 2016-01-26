use std::cmp::{min, max};
use std::fs::File;
use std::io::{Result, Seek, SeekFrom, Write};
use std::mem::size_of;
use std::slice;

use orbital::Color;

pub struct Display {
    file: File,
    width: i32,
    height: i32,
    data: Box<[Color]>,
}

impl Display {
    pub fn new() -> Result<Display> {
        let mut file = try!(File::open("display:"));

        let path = try!(file.path()).to_string();
        let res = path.split(":").nth(1).unwrap_or("");
        let width = res.split("x").nth(0).unwrap_or("").parse::<i32>().unwrap_or(0);
        let height = res.split("x").nth(1).unwrap_or("").parse::<i32>().unwrap_or(0);

        let mut data: Vec<Color> = Vec::new();
        {
            let size = width as usize * height as usize;
            while data.len() < size {
                data.push(Color::rgb(0, 0, 0));
            }
        }

        Ok(Display {
            file: file,
            width: width,
            height: height,
            data: data.into_boxed_slice()
        })
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color) {
        for py in min(self.height as i32, max(0, y)) .. min(self.height, max(0, y + h)) {
            let row = py * self.width;
            for px in min(self.width, max(0, x)) .. min(self.width, max(0, x + w)) {
                self.data[(row + px) as usize] = color;
            }
        }
    }

    pub fn set(&mut self, color: Color) {
        for c in self.data.iter_mut() {
            *c = color
        }
    }

    pub fn flip(&mut self) -> Result<()> {
        try!(self.file.write(unsafe { & slice::from_raw_parts(self.data.as_ptr() as *const u8, self.data.len() * size_of::<Color>()) }));
        Ok(())
    }
}
