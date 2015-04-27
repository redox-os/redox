use core::cmp::min;
use core::cmp::max;
use core::str::StrExt;

use graphics::color::*;
use graphics::point::*;
use graphics::size::*;

const VBEMODEINFOLOCATION: usize = 0x5200;

#[derive(Copy, Clone)]
pub struct VBEModeInfo {
	attributes: u16,
	win_a: u8,
	win_b: u8,
	granularity: u16,
	winsize: u16,
	segment_a: u16,
	segment_b: u16,
	winfuncptr: u32,
	bytesperscanline: u16,
	xresolution: u16,
	yresolution: u16,
	xcharsize: u8,
	ycharsize: u8,
	numberofplanes: u8,
	bitsperpixel: u8,
	numberofbanks: u8,
	memorymodel: u8,
	banksize: u8,
	numberofimagepages: u8,
	unused: u8,
	redmasksize: u8,
	redfieldposition: u8,
	greenmasksize: u8,
	greenfieldposition: u8,
	bluemasksize: u8,
	bluefieldposition: u8,
	rsvdmasksize: u8,
	rsvdfieldposition: u8,
	directcolormodeinfo: u8,
	physbaseptr: u32,
	offscreenmemoryoffset: u32,
	offscreenmemsize: u16
}

pub static mut FONT_LOCATION: usize = 0x0;

pub struct Display {
	mode_info: VBEModeInfo,
	bytesperrow: usize,
	bytesperpixel: usize,
	pub size: Size
}

const OFFSCREENLOCATION: usize = 0x400000;

impl Display {
	pub fn new() -> Display {
        unsafe{
            let mode_info = *(VBEMODEINFOLOCATION as *const VBEModeInfo);
            Display {
                mode_info: mode_info,
                bytesperrow: mode_info.bytesperscanline as usize,
                bytesperpixel: mode_info.bitsperpixel as usize/8,
                size: Size {
                    width: mode_info.xresolution as u32,
                    height: mode_info.yresolution as u32
                }
            }
        }
	}
	
	#[inline(always)]
	unsafe fn fast_pixel(pixel_ptr: usize, color: Color){
        if color.a == 255 {
            *(pixel_ptr as *mut u8) = color.r;
            *((pixel_ptr + 1) as *mut u8) = color.g;
            *((pixel_ptr + 2) as *mut u8) = color.b;
        }else{
            let r = color.r as usize;
            let g = color.g as usize;
            let b = color.b as usize;
            let a = color.a as usize;
            
            let o_r = *(pixel_ptr as *const u8) as usize;
            let o_g = *((pixel_ptr + 1) as *const u8) as usize;
            let o_b = *((pixel_ptr + 2) as *const u8) as usize;
            let o_a = 255 - a;
            
            *(pixel_ptr as *mut u8) = ((o_r * o_a + r * a)/255) as u8;
            *((pixel_ptr + 1) as *mut u8) = ((o_g * o_a + g * a)/255) as u8;
            *((pixel_ptr + 2) as *mut u8) = ((o_b * o_a + b * a)/255) as u8;
        }
	}

	pub fn pixel(&self, point: Point, color: Color){
        unsafe{
            if color.a > 0 {
                if point.x >= 0 && point.x < self.size.width as i32 && point.y >= 0 && point.y < self.size.height as i32 {
                    let pixel_ptr: usize = OFFSCREENLOCATION + point.y as usize * self.bytesperrow + point.x as usize * self.bytesperpixel;
                    
                    Display::fast_pixel(pixel_ptr, color);
                }
			}
		}
	}

	pub fn rect(&self, point: Point, size: Size, color: Color){
        if color.a > 0 {
            let start_y = max(0, min(self.size.height as i32 - 1, point.y)) as usize;
            let end_y = max(0, min(self.size.height as i32 - 1, point.y + size.height as i32)) as usize;
            
            let start_x = max(0, min(self.size.width as i32 - 1, point.x)) as usize;
            let end_x = max(0, min(self.size.width as i32 - 1, point.x + size.width as i32)) as usize;
        
            for y in start_y..end_y {
                let row_ptr: usize = OFFSCREENLOCATION + y * self.bytesperrow;
                for x in start_x..end_x {
                    let pixel_ptr: usize = row_ptr + x * self.bytesperpixel;
                
                    unsafe{
                        Display::fast_pixel(pixel_ptr, color);
                    }
                }
			}
		}
	}

	pub fn clear(&self, color:Color){
        unsafe {
            let mut i = 0;
            while i < self.mode_info.yresolution as usize * self.mode_info.bytesperscanline as usize {
				*((OFFSCREENLOCATION + i) as *mut u8) = color.r;
				*((OFFSCREENLOCATION + i + 1) as *mut u8) = color.g;
				*((OFFSCREENLOCATION + i + 2) as *mut u8) = color.b;
                i += 3;
			}
		}
	}

	pub unsafe fn char_bitmap(&self, point: Point, bitmap_location: *const u8, color: Color){
        for row in 0..16 {
            let row_data = *((bitmap_location as usize + row) as *const u8);
            for col in 0..8 {
                let pixel = (row_data >> (7 - col)) & 1;
                if pixel > 0 {
                    self.pixel(Point::new(point.x + col, point.y + row as i32), color);
                }
            }
        }
	}

	pub fn char(&self, point: Point, character: char, color: Color){
        unsafe{
            if FONT_LOCATION > 0 {
                self.char_bitmap(point, (FONT_LOCATION + 16*(character as usize)) as *const u8, color);
            }
        }
	}

	pub fn text(&self, point: Point, text: &str, color: Color){
		let mut cursor = Point::new(point.x, point.y);
		for character in text.chars() {
			self.char(cursor, character, color);
			cursor.x += 8;
		}
	}

    pub unsafe fn c_text(&self, point: Point, c_text: *const u8, color: Color){
        let mut cursor = Point::new(point.x, point.y);
        for i in 0..(self.mode_info.xresolution as usize - point.x as usize)/8 {
            let character = *((c_text as usize + i) as *const u8);
            if character == 0 {
                break;
            }
            self.char(cursor, character as char, color);
            cursor.x += 8;
        }
    }

	pub fn copy(&self){
        unsafe{
            let mut i = 0;
            while i < self.mode_info.yresolution as usize * self.mode_info.bytesperscanline as usize {
                let data: u64 = *((OFFSCREENLOCATION + i) as *const u64);
                *((self.mode_info.physbaseptr as usize + i) as *mut u64) = data;
                i += 8;
            }
		}
	}
}
