use std::cmp::{min, max};

use super::Color;

pub struct ImageRoi<'a> {
    x1: i32,
    x2: i32,
    y1: i32,
    y2: i32,
    image: &'a mut Image
}

impl<'a> ImageRoi<'a> {
    pub fn width(&self) -> i32 {
        self.x2 - self.x1
    }

    pub fn height(&self) -> i32 {
        self.y2 - self.y1
    }

    pub fn roi(&mut self, x: i32, y: i32, w: i32, h: i32) -> ImageRoi {
        let start_x = min(self.x2, max(self.x1, x));
        let end_x = min(self.x2, start_x + max(0, w));
        let start_y = min(self.y2, max(self.y1, y));
        let end_y = min(self.y2, start_y + max(0, h));

        self.image.roi(start_x, start_y, end_x - start_x, end_y - start_y)
    }

    pub fn set(&mut self, color: Color) {
        for y in self.y1..self.y2 {
            let row = y * self.image.width();
            for x in self.x1..self.x2 {
                self.image.data[(row + x) as usize] = color;
            }
        }
    }

    pub fn blit(&mut self, other: &ImageRoi) {
        for y in 0..min(self.height(), other.height()) {
            let row = (self.y1 + y) * self.image.width();
            let other_row = (other.y1 + y) * other.image.width();
            for x in 0..min(self.width(), other.width()) {
                self.image.data[(row + self.x1 + x) as usize] = other.image.data[(other_row + other.x1 + x) as usize];
            }
        }
    }

    pub fn blend(&mut self, other: &ImageRoi) {
        for y in 0..min(self.height(), other.height()) {
            let row = (self.y1 + y) * self.image.width();
            let other_row = (other.y1 + y) * other.image.width();
            for x in 0..min(self.width(), other.width()) {
                let new = other.image.data[(other_row + other.x1 + x) as usize].data;

                let alpha = (new >> 24) & 0xFF;
                if alpha > 0 {
                    let old = &mut self.image.data[(row + self.x1 + x) as usize].data;
                    if alpha >= 255 {
                        *old = new;
                    } else {
                        let n_r = (((new >> 16) & 0xFF) * alpha) >> 8;
                        let n_g = (((new >> 8) & 0xFF) * alpha) >> 8;
                        let n_b = ((new & 0xFF) * alpha) >> 8;

                        let n_alpha = 255 - alpha;
                        let o_r = (((*old >> 16) & 0xFF) * n_alpha) >> 8;
                        let o_g = (((*old >> 8) & 0xFF) * n_alpha) >> 8;
                        let o_b = ((*old & 0xFF) * n_alpha) >> 8;

                        *old = ((o_r << 16) | (o_g << 8) | o_b) + ((n_r << 16) | (n_g << 8) | n_b);
                    }
                }
            }
        }
    }
}

pub struct Image {
    w: i32,
    h: i32,
    data: Box<[Color]>
}

impl Image {
    pub fn new(width: i32, height: i32) -> Image {
        Image::from_color(width, height, Color::rgb(0, 0, 0))
    }

    pub fn from_color(width: i32, height: i32, color: Color) -> Image {
        let mut data: Vec<Color> = Vec::new();
        {
            let size = width as usize * height as usize;
            while data.len() < size {
                data.push(color);
            }
        }

        Image::from_data(width, height, data.into_boxed_slice())
    }

    pub fn from_data(width: i32, height: i32, data: Box<[Color]>) -> Image {
        Image {
            w: width,
            h: height,
            data: data
        }
    }

    pub fn width(&self) -> i32 {
        self.w
    }

    pub fn height(&self) -> i32 {
        self.h
    }

    pub fn data(&self) -> &[Color] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [Color] {
        &mut self.data
    }

    pub fn as_roi(&mut self) -> ImageRoi {
        ImageRoi {
            x1: 0,
            x2: self.w,
            y1: 0,
            y2: self.h,
            image: self
        }
    }

    pub fn roi(&mut self, x: i32, y: i32, w: i32, h: i32) -> ImageRoi {
        let start_x = min(self.width(), max(0, x));
        let end_x = min(self.width(), start_x + max(0, w));
        let start_y = min(self.height(), max(0, y));
        let end_y = min(self.height(), start_y + max(0, h));

        ImageRoi {
            x1: start_x,
            x2: end_x,
            y1: start_y,
            y2: end_y,
            image: self
        }
    }
}
