use std::{cmp, mem};

use system::graphics::{fast_copy, fast_set};

use super::{Color, Rect};

pub struct ImageRoiRows<'a> {
    rect: Rect,
    image: &'a Image,
    i: i32,
}

impl<'a> Iterator for ImageRoiRows<'a> {
    type Item = &'a [u32];
    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.rect.height() {
            let start = (self.rect.top() + self.i) * self.image.width() + self.rect.left();
            let end = start + self.rect.width();
            self.i += 1;
            Some(& self.image.data[start as usize .. end as usize])
        } else {
            None
        }
    }
}

pub struct ImageRoiRowsMut<'a> {
    rect: Rect,
    image: &'a mut Image,
    i: i32,
}

impl<'a> Iterator for ImageRoiRowsMut<'a> {
    type Item = &'a mut [u32];
    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.rect.height() {
            let start = (self.rect.top() + self.i) * self.image.width() + self.rect.left();
            let end = start + self.rect.width();
            self.i += 1;
            // it does not appear to be possible to do this in safe rust
            Some(unsafe { mem::transmute(&mut self.image.data[start as usize .. end as usize]) })
        } else {
            None
        }
    }
}

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

    pub fn rows(&'a self) -> ImageRoiRows<'a> {
        ImageRoiRows {
            rect: self.rect,
            image: self.image,
            i: 0
        }
    }

    pub fn rows_mut(&'a mut self) -> ImageRoiRowsMut<'a> {
        ImageRoiRowsMut {
            rect: self.rect,
            image: self.image,
            i: 0
        }
    }

    pub fn blend(&'a mut self, other: &ImageRoi) {
        for (mut self_row, other_row) in self.rows_mut().zip(other.rows()) {
            for(mut old, new) in self_row.iter_mut().zip(other_row.iter()) {
                let alpha = (*new >> 24) & 0xFF;
                if alpha >= 255 {
                    *old = *new;
                } else if alpha > 0 {
                    let n_r = (((*new >> 16) & 0xFF) * alpha) >> 8;
                    let n_g = (((*new >> 8) & 0xFF) * alpha) >> 8;
                    let n_b = ((*new & 0xFF) * alpha) >> 8;

                    let n_alpha = 255 - alpha;

                    let o_r = (((*old >> 16) & 0xFF) * n_alpha) >> 8;
                    let o_g = (((*old >> 8) & 0xFF) * n_alpha) >> 8;
                    let o_b = ((*old & 0xFF) * n_alpha) >> 8;

                    *old = ((o_r << 16) | (o_g << 8) | o_b) + ((n_r << 16) | (n_g << 8) | n_b);
                }
            }
        }
    }

    pub fn blit(&'a mut self, other: &ImageRoi) {
        for (mut self_row, other_row) in self.rows_mut().zip(other.rows()) {
            let len = cmp::min(self_row.len(), other_row.len());
            unsafe { fast_copy(self_row.as_mut_ptr() as *mut u32, other_row.as_ptr() as *const u32, len); }
        }
    }

    pub fn set(&'a mut self, color: Color) {
        let new = color.data;

        let alpha = (new >> 24) & 0xFF;
        if alpha >= 255 {
            for mut self_row in self.rows_mut() {
                unsafe { fast_set(self_row.as_mut_ptr() as *mut u32, new, self_row.len()); }
            }
        } else if alpha > 0 {
            let n_r = (((new >> 16) & 0xFF) * alpha) >> 8;
            let n_g = (((new >> 8) & 0xFF) * alpha) >> 8;
            let n_b = ((new & 0xFF) * alpha) >> 8;

            let n_alpha = 255 - alpha;

            for mut self_row in self.rows_mut() {
                for mut old in self_row.iter_mut() {
                    let o_r = (((*old >> 16) & 0xFF) * n_alpha) >> 8;
                    let o_g = (((*old >> 8) & 0xFF) * n_alpha) >> 8;
                    let o_b = ((*old & 0xFF) * n_alpha) >> 8;

                    *old = ((o_r << 16) | (o_g << 8) | o_b) + ((n_r << 16) | (n_g << 8) | n_b);
                }
            }
        }
    }
}

pub struct Image {
    w: i32,
    h: i32,
    data: Box<[u32]>
}

impl Image {
    pub fn new(width: i32, height: i32) -> Image {
        Image::from_color(width, height, Color::rgb(0, 0, 0))
    }

    pub fn from_color(width: i32, height: i32, color: Color) -> Image {
        Image::from_data(width, height, vec![color.data; width as usize * height as usize].into_boxed_slice())
    }

    pub fn from_data(width: i32, height: i32, data: Box<[u32]>) -> Image {
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

    pub fn data(&self) -> &[u32] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u32] {
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
