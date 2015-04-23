use core::str::StrExt;

use graphics::color::*;
use graphics::point::*;
use graphics::size::*;

const VBEMODEINFOLOCATION: usize = 0x5200;

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
	mode_info: *const VBEModeInfo
}

const OFFSCREENLOCATION: usize = 0x400000;

impl Display {
	pub fn new() -> Display {
        Display { mode_info: (VBEMODEINFOLOCATION as *const VBEModeInfo) }
	}

	pub fn size(&self) -> Size {
        unsafe {
            Size { width:(*self.mode_info).xresolution as u32, height:(*self.mode_info).yresolution as u32 }
		}
	}

	pub fn pixel(&self, point: Point, color: Color){
        unsafe{
            if color.a > 0 {
                if point.x >= 0 && point.x < (*self.mode_info).xresolution as i32 && point.y >= 0 &&  point.y < (*self.mode_info).yresolution as i32 {
                    let pixelptr: usize = OFFSCREENLOCATION + point.y as usize * (*self.mode_info).bytesperscanline as usize + point.x as usize * 3;
                    if color.a == 255 {
                        *(pixelptr as *mut u8) = color.r;
                        *((pixelptr + 1) as *mut u8) = color.g;
                        *((pixelptr + 2) as *mut u8) = color.b;
                    }else{
                        let r = color.r as usize;
                        let g = color.g as usize;
                        let b = color.b as usize;
                        let a = color.a as usize;
                        
                        let o_r = *(pixelptr as *const u8) as usize;
                        let o_g = *((pixelptr + 1) as *const u8) as usize;
                        let o_b = *((pixelptr + 2) as *const u8) as usize;
                        let o_a = 255 - a;
                        
                        *(pixelptr as *mut u8) = ((o_r * o_a + r * a)/255) as u8;
                        *((pixelptr + 1) as *mut u8) = ((o_g * o_a + g * a)/255) as u8;
                        *((pixelptr + 2) as *mut u8) = ((o_b * o_a + b * a)/255) as u8;
                    }
                }
			}
		}
	}

	pub fn rect(&self, point: Point, size: Size, color: Color){
		for y in point.y..point.y + size.height as i32 {
			for x in point.x..point.x + size.width as i32 {
				self.pixel(Point::new(x, y), color);
			}
		}
	}

	pub fn clear(&self, color:Color){
        unsafe {
            let mut i = 0;
            while i < (*self.mode_info).yresolution as usize * (*self.mode_info).bytesperscanline as usize {
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
        for i in 0..((*self.mode_info).xresolution as usize - point.x as usize)/8 {
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
            while i < (*self.mode_info).yresolution as usize * (*self.mode_info).bytesperscanline as usize {
                let data: u64 = *((OFFSCREENLOCATION + i) as *const u64);
                *((((*self.mode_info).physbaseptr as usize) + i) as *mut u64) = data;
                i += 8;
            }
		}
	}
}
