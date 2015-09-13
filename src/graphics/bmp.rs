use core::ops::Drop;
use core::ptr;

use common::debug::*;
use common::memory::*;

use graphics::color::*;
use graphics::size::*;

pub struct BMP {
    pub data: usize,
    pub size: Size
}

impl BMP {
    pub fn new() -> BMP {
        BMP {
            data: 0,
            size: Size { width: 0, height: 0}
        }
    }

    pub fn from_data(file_data: usize) -> BMP {
        let data;
        let size;
        unsafe {
            if file_data > 0
                && *(file_data as *const u8) == 'B' as u8
                && ptr::read((file_data + 1) as *const u8) == 'M' as u8
            {
                let file_size = ptr::read((file_data + 0x2) as *const u32) as usize;
                let offset = ptr::read((file_data + 0xA) as *const u32) as usize;
                let header_size = ptr::read((file_data + 0xE) as *const u32) as usize;
                let width = ptr::read((file_data + 0x12) as *const u32) as usize;
                let height = ptr::read((file_data + 0x16) as *const u32) as usize;
                let depth = ptr::read((file_data + 0x1C) as *const u16) as usize;

                let bytes = (depth + 7)/8;
                let row_bytes = (depth * width + 31)/32 * 4;

                let mut red_mask = 0xFF0000;
                let mut green_mask = 0xFF00;
                let mut blue_mask = 0xFF;
                let mut alpha_mask = 0xFF000000;
                if ptr::read((file_data + 0x1E) as *const u32) == 3 {
                    red_mask = ptr::read((file_data + 0x36) as *const u32) as usize;
                    green_mask = ptr::read((file_data + 0x3A) as *const u32) as usize;
                    blue_mask = ptr::read((file_data + 0x3E) as *const u32) as usize;
                    alpha_mask = ptr::read((file_data + 0x42) as *const u32) as usize;
                }

                let mut red_shift = 0;
                while red_mask > 0 && red_shift < 32 && (red_mask >> red_shift) & 1 == 0 {
                    red_shift += 1;
                }

                let mut green_shift = 0;
                while green_mask > 0 && green_shift < 32 && (green_mask >> green_shift) & 1 == 0 {
                    green_shift += 1;
                }

                let mut blue_shift = 0;
                while blue_mask > 0 && blue_shift < 32 && (blue_mask >> blue_shift) & 1 == 0 {
                    blue_shift += 1;
                }

                let mut alpha_shift = 0;
                while alpha_mask > 0 && alpha_shift < 32 && (alpha_mask >> alpha_shift) & 1 == 0 {
                    alpha_shift += 1;
                }

                data = alloc(width * height * 4);
                size = Size {
                    width: width,
                    height: height
                };
                for y in 0..height {
                    for x in 0..width {
                        let pixel_offset = offset + (height - y - 1) * row_bytes + x * bytes;

                        let pixel_data;
                        if pixel_offset < file_size {
                            pixel_data = ptr::read((file_data + pixel_offset) as *const u32) as usize;
                        }else{
                            pixel_data = 0;
                        }

                        let red = ((pixel_data & red_mask) >> red_shift) as u8;
                        let green = ((pixel_data & green_mask) >> green_shift) as u8;
                        let blue = ((pixel_data & blue_mask) >> blue_shift) as u8;
                        if bytes == 3 {
                            ptr::write((data + (y*width + x)*4) as *mut u32, Color::new(red, green, blue).data);
                        }else if bytes == 4 {
                            let alpha = ((pixel_data & alpha_mask) >> alpha_shift) as u8;
                            ptr::write((data + (y*width + x)*4) as *mut u32, Color::alpha(red, green, blue, alpha).data);
                        }
                    }
                }
            }else{
                data = 0;
                size = Size {
                    width: 0,
                    height: 0
                };
            }
        }

        return BMP {
            data: data,
            size: size
        };
    }
}

impl Drop for BMP {
    fn drop(&mut self){
        unsafe {
            if self.data > 0 {
                unalloc(self.data);
                self.data = 0;
                self.size = Size {
                    width: 0,
                    height: 0
                };
            }
        }
    }
}
