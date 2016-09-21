use std::cmp;

use primitive::{fast_set, fast_set64, fast_copy64};

static FONT: &'static [u8] = include_bytes!("../../../res/unifont.font");

/// A display
pub struct Display {
    pub width: usize,
    pub height: usize,
    pub onscreen: &'static mut [u32],
    pub offscreen: &'static mut [u32],
}

impl Display {
    pub fn new(width: usize, height: usize, onscreen: &'static mut [u32], offscreen: &'static mut [u32]) -> Display {
        Display {
            width: width,
            height: height,
            onscreen: onscreen,
            offscreen: offscreen,
        }
    }

    /// Draw a rectangle
    pub fn rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        let start_y = cmp::min(self.height - 1, y);
        let end_y = cmp::min(self.height, y + h);

        let start_x = cmp::min(self.width - 1, x);
        let len = cmp::min(self.width, x + w) - start_x;

        let mut offscreen_ptr = self.offscreen.as_mut_ptr() as usize;
        let mut onscreen_ptr = self.onscreen.as_mut_ptr() as usize;

        let stride = self.width * 4;

        let offset = y * stride + start_x * 4;
        offscreen_ptr += offset;
        onscreen_ptr += offset;

        let mut rows = end_y - start_y;
        while rows > 0 {
            unsafe {
                fast_set(offscreen_ptr as *mut u32, color, len);
                fast_set(onscreen_ptr as *mut u32, color, len);
            }
            offscreen_ptr += stride;
            onscreen_ptr += stride;
            rows -= 1;
        }
    }

    /// Draw a character
    fn char(&mut self, x: usize, y: usize, character: char, color: u32) {
        if x + 8 <= self.width && y + 16 <= self.height {
            let mut font_i = 16 * (character as usize);
            let font_end = font_i + 16;
            if font_end <= FONT.len() {
                let mut offscreen_ptr = self.offscreen.as_mut_ptr() as usize;
                let mut onscreen_ptr = self.onscreen.as_mut_ptr() as usize;

                let stride = self.width * 4;

                let offset = y * stride + x * 4;
                offscreen_ptr += offset;
                onscreen_ptr += offset;

                while font_i < font_end {
                    let mut row_data = FONT[font_i];
                    let mut col = 8;
                    while col > 0 {
                        col -= 1;
                        if row_data & 1 == 1 {
                            unsafe {
                                *((offscreen_ptr + col * 4) as *mut u32) = color;
                            }
                        }
                        row_data = row_data >> 1;
                    }

                    unsafe {
                        fast_copy64(onscreen_ptr as *mut u64, offscreen_ptr as *const u64, 4);
                    }

                    offscreen_ptr += stride;
                    onscreen_ptr += stride;
                    font_i += 1;
                }
            }
        }
    }

    /// Scroll display
    pub fn scroll(&mut self, rows: usize, color: u32) {
        let data = (color as u64) << 32 | color as u64;

        let width = self.width/2;
        let height = self.height;
        if rows > 0 && rows < height {
            let off1 = rows * width;
            let off2 = height * width - off1;
            unsafe {
                let data_ptr = self.offscreen.as_mut_ptr() as *mut u64;
                fast_copy64(data_ptr, data_ptr.offset(off1 as isize), off2);
                fast_set64(data_ptr.offset(off2 as isize), data, off1);

                fast_copy64(self.onscreen.as_mut_ptr() as *mut u64, data_ptr, off1 + off2);
            }
        }
    }
}
