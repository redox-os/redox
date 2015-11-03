use super::color::Color;

use core::{mem};
use fs::File;
use io::*;
use string::String;
use vec::Vec;
use super::display::Display;
use super::point::Point;
use super::size::Size;

// TODO: Follow naming convention
/// A bitmap
pub struct BMPFile {
    /// The bitmap width
    w: usize,
    /// The bitmap height
    h: usize,
    /// The data of the bitmap
    pub data: Vec<Color>,
}

impl BMPFile {
    /// Create a new bitmap
    pub fn new(width: usize, height: usize) -> Self {
        BMPFile {
            w: width,
            h: height,
            data: Vec::new(),
        }
    }

    /// Create a new empty bitmap
    pub fn default() -> Self {
        Self::new(0, 0)
    }

    /// Load a bitmap from a file path
    pub fn load(path: &str) -> Self {
        match File::open(path) {
            Some(mut f) => {
                let mut data = Vec::new();
                f.read_to_end(&mut data);
                Self::from_data(&data)
            },
            None => Self::new(0,0),
        }
    }

    /// Create a bitmap from some data
    pub fn from_data(file_data: &[u8]) -> Self {
        let get = |i: usize| -> u8 {
            match file_data.get(i) {
                Some(byte) => *byte,
                None => 0,
            }
        };

        let getw = |i: usize| -> u16 { (get(i) as u16) + ((get(i + 1) as u16) << 8) };

        let getd = |i: usize| -> u32 {
            (get(i) as u32) + ((get(i + 1) as u32) << 8) + ((get(i + 2) as u32) << 16) +
            ((get(i + 3) as u32) << 24)
        };

        let gets = |start: usize, len: usize| -> String {
            (start..start + len).map(|i| get(i) as char).collect::<String>()
        };

        let mut ret: BMPFile;

        if gets(0, 2) == "BM" {
            //let file_size = getd(2);
            let offset = getd(0xA);
            //let header_size = getd(0xE);
            let width = getd(0x12);
            let height = getd(0x16);
            let depth = getw(0x1C) as u32;

            let bytes = (depth + 7) / 8;
            let row_bytes = (depth * width + 31) / 32 * 4;

            let mut blue_mask = 0xFF;
            let mut green_mask = 0xFF00;
            let mut red_mask = 0xFF0000;
            let mut alpha_mask = 0xFF000000;
            if getd(0x1E) == 3 {
                red_mask = getd(0x36);
                green_mask = getd(0x3A);
                blue_mask = getd(0x3E);
                alpha_mask = getd(0x42);
            }

            let mut blue_shift = 0;
            while blue_mask > 0 && blue_shift < 32 && (blue_mask >> blue_shift) & 1 == 0 {
                blue_shift += 1;
            }

            let mut green_shift = 0;
            while green_mask > 0 && green_shift < 32 && (green_mask >> green_shift) & 1 == 0 {
                green_shift += 1;
            }

            let mut red_shift = 0;
            while red_mask > 0 && red_shift < 32 && (red_mask >> red_shift) & 1 == 0 {
                red_shift += 1;
            }

            let mut alpha_shift = 0;
            while alpha_mask > 0 && alpha_shift < 32 && (alpha_mask >> alpha_shift) & 1 == 0 {
                alpha_shift += 1;
            }

            ret = Self::new(width as usize, height as usize);

            for y in 0..height {
                for x in 0..width {
                    let pixel_offset = offset + (height - y - 1) * row_bytes + x * bytes;

                    let pixel_data = getd(pixel_offset as usize);
                    let red = ((pixel_data & red_mask) >> red_shift) as u8;
                    let green = ((pixel_data & green_mask) >> green_shift) as u8;
                    let blue = ((pixel_data & blue_mask) >> blue_shift) as u8;
                    let alpha = ((pixel_data & alpha_mask) >> alpha_shift) as u8;
                    if bytes == 3 {
                        ret.data.push(Color::rgb(red,green,blue));
                    } else if bytes == 4 {
                        ret.data.push(Color::rgba(red,green,blue,alpha));
                    }
                }
            }
        } else {
            ret = Self::default();
        }

        ret
    }

    /// Convert to slice for drawing
    pub fn as_slice(&self) -> &[Color] {
        &self.data
    }

    pub fn width(&self) -> usize {
        self.w
    }

    pub fn height(&self) -> usize {
        self.h
    }

    pub fn size(&self) -> Size {
        Size{ width: self.w, height: self.h }
    }

    pub fn data_ptr(&self) -> *const u32 {
        let data : &[u32] = unsafe {mem::transmute::<&[Color],&[u32]>(&self.data) };
        data.as_ptr()
    }
    /// Draw the bitmap to the screen
    pub fn draw(&self, display: &Display, point: Point) {
        unsafe {
            display.image_alpha(point, self.data_ptr(), self.size());
        }
    }
}
