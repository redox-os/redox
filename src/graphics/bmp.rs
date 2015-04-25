use core::ops::Drop;

use common::memory::*;

use graphics::color::*;
use graphics::point::*;
use graphics::size::*;

pub struct BMP {
    pub data: usize,
    pub size: Size
}

impl BMP {
    pub unsafe fn new() -> BMP {
        BMP {
            data: 0,
            size: Size { width: 0, height: 0}
        }
    }

    pub unsafe fn from_data(file_data: usize) -> BMP {
        let data;
        let size;
        if file_data > 0
            && *(file_data as *const u8) == 'B' as u8
            && *((file_data + 1) as *const u8) == 'M' as u8
        {
            let file_size = *((file_data + 0x2) as *const u32) as usize;
            let offset = *((file_data + 0xA) as *const u32) as usize;
            let width = *((file_data + 0x12) as *const u32) as usize;
            let height = *((file_data + 0x16) as *const u32) as usize;
            let depth = *((file_data + 0x1C) as *const u16) as usize;
            
            let bytes = (depth + 7)/8;
            let row_bytes = (depth * width + 31)/32 * 4;
            
            data = alloc(width * height * 4);
            size = Size {
                width: width as u32,
                height: height as u32
            };
            for y in 0..height {
                for x in 0..width {
                    let pixel_offset = offset + (height - y - 1) * row_bytes + x * bytes;
                    
                    let pixel_data;
                    if pixel_offset < file_size {
                        pixel_data = *((file_data + pixel_offset) as *const u32);
                    }else{
                        pixel_data = 0;
                    }
                    *((data + (y*width + x)*4) as *mut Color) = Color::new(pixel_data as u8, (pixel_data >> 8) as u8, (pixel_data >> 16) as u8);
                }
            }
        }else{
            data = 0;
            size = Size {
                width: 0,
                height: 0
            };
        }
    
        return BMP {
            data: data,
            size: size
        };
    }
    
    pub unsafe fn pixel(&self, point: Point) -> Color {
        if point.x >= 0
            && point.x < self.size.width as i32
            && point.y >= 0
            && point.y < self.size.height as i32
        {
            return *((self.data + (point.y as usize*self.size.width as usize + point.x as usize)*4) as *const Color);
        }else{
            return Color::new(0, 0, 0);
        }
    }
}

impl Drop for BMP {
    fn drop(&mut self){
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