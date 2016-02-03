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

    pub fn blend(&mut self, other: &ImageRoi) {
        let start_y = max(-self.y1, -other.y1);
        let end_y = min(min(self.image.height(), self.y2) - self.y1, min(other.image.height(), other.y2) - other.y1);
        let start_x = max(-self.x1, -other.x1);
        let end_x = min(min(self.image.width(), self.x2) - self.x1, min(other.image.width(), other.x2) - other.x1);

        for y in start_y..end_y {
            let row = (self.y1 + y) * self.image.width();
            let other_row = (other.y1 + y) * other.image.width();
            for x in start_x..end_x {
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

    pub fn set(&mut self, color: Color) {
        let new = color.data;

        let alpha = (new >> 24) & 0xFF;
        if alpha > 0 {
            if alpha >= 255 {
                for y in max(0, self.y1) .. min(self.image.height(), self.y2) {
                    let row = y * self.image.width();
                    for x in max(0, self.x1) .. min(self.image.width(), self.x2) {
                        self.image.data[(row + x) as usize].data = new;
                    }
                }
            } else {
                let n_r = (((new >> 16) & 0xFF) * alpha) >> 8;
                let n_g = (((new >> 8) & 0xFF) * alpha) >> 8;
                let n_b = ((new & 0xFF) * alpha) >> 8;

                let n_alpha = 255 - alpha;

                for y in max(0, self.y1) .. min(self.image.height(), self.y2) {
                    let row = y * self.image.width();
                    for x in max(0, self.x1) .. min(self.image.width(), self.x2) {
                        let old = &mut self.image.data[(row + x) as usize].data;

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
        ImageRoi {
            x1: x,
            x2: x + w,
            y1: y,
            y2: y + h,
            image: self
        }
    }
}
