use std::cmp::{min, max};

use super::{Color, Rect};

pub struct ImageRoi<'a> {
    rect: Rect,
    image: &'a mut Image
}

impl<'a> ImageRoi<'a> {
    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn left(&self) -> i32 {
        self.rect.left()
    }

    pub fn right(&self) -> i32 {
        self.rect.right()
    }

    pub fn top(&self) -> i32 {
        self.rect.top()
    }

    pub fn bottom(&self) -> i32 {
        self.rect.bottom()
    }

    pub fn width(&self) -> i32 {
        self.rect.width()
    }

    pub fn height(&self) -> i32 {
        self.rect.height()
    }

    pub fn blend(&'a mut self, other: &ImageRoi) -> &'a mut ImageRoi {
        let start_y = max(-self.top(), -other.top());
        let end_y = min(min(self.image.height(), self.bottom()) - self.top(), min(other.image.height(), other.bottom()) - other.top());
        let start_x = max(-self.left(), -other.left());
        let end_x = min(min(self.image.width(), self.right()) - self.left(), min(other.image.width(), other.right()) - other.left());

        for y in start_y..end_y {
            let row = (self.top() + y) * self.image.width();
            let other_row = (other.top() + y) * other.image.width();
            for x in start_x..end_x {
                let new = other.image.data[(other_row + other.left() + x) as usize].data;

                let alpha = (new >> 24) & 0xFF;
                if alpha > 0 {
                    let old = &mut self.image.data[(row + self.left() + x) as usize].data;
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

        self
    }

    pub fn set(&'a mut self, color: Color) -> &'a mut ImageRoi {
        let new = color.data;

        let alpha = (new >> 24) & 0xFF;
        if alpha > 0 {
            if alpha >= 255 {
                for y in max(0, self.top()) .. min(self.image.height(), self.bottom()) {
                    let row = y * self.image.width();
                    for x in max(0, self.left()) .. min(self.image.width(), self.right()) {
                        self.image.data[(row + x) as usize].data = new;
                    }
                }
            } else {
                let n_r = (((new >> 16) & 0xFF) * alpha) >> 8;
                let n_g = (((new >> 8) & 0xFF) * alpha) >> 8;
                let n_b = ((new & 0xFF) * alpha) >> 8;

                let n_alpha = 255 - alpha;

                for y in max(0, self.top()) .. min(self.image.height(), self.bottom()) {
                    let row = y * self.image.width();
                    for x in max(0, self.left()) .. min(self.image.width(), self.right()) {
                        let old = &mut self.image.data[(row + x) as usize].data;

                        let o_r = (((*old >> 16) & 0xFF) * n_alpha) >> 8;
                        let o_g = (((*old >> 8) & 0xFF) * n_alpha) >> 8;
                        let o_b = ((*old & 0xFF) * n_alpha) >> 8;

                        *old = ((o_r << 16) | (o_g << 8) | o_b) + ((n_r << 16) | (n_g << 8) | n_b);
                    }
                }
            }
        }

        self
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
            rect: Rect::new(0, 0, self.w, self.h),
            image: self
        }
    }

    pub fn roi(&mut self, rect: &Rect) -> ImageRoi {
        ImageRoi {
            rect: *rect,
            image: self
        }
    }
}
