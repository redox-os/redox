use core::str::StrExt;

use graphics::color::*;
use graphics::point::*;
use graphics::size::*;
use graphics::window::*;

const VBEMODEINFOLOCATION: u32 = 0x5200;

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

pub static mut FONT_LOCATION: u32 = 0x0;

pub struct Display {
	mode_info: *const VBEModeInfo
}

const OFFSCREENLOCATION: u32 = 0x400000;

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
            if point.x >= 0 && point.x < (*self.mode_info).xresolution as i32 && point.y >= 0 &&  point.y < (*self.mode_info).yresolution as i32 {
                let pixelptr: u32 = (OFFSCREENLOCATION as u32) + point.y as u32 * (*self.mode_info).bytesperscanline as u32 + point.x as u32 * 3;
				*(pixelptr as *mut u8) = color.r;
				*((pixelptr + 1) as *mut u8) = color.g;
				*((pixelptr + 2) as *mut u8) = color.b;
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
            while i < (*self.mode_info).yresolution as u32 * (*self.mode_info).bytesperscanline as u32 {
				*(((OFFSCREENLOCATION as u32) + i) as *mut u8) = color.r;
				*(((OFFSCREENLOCATION as u32) + i + 1) as *mut u8) = color.g;
				*(((OFFSCREENLOCATION as u32) + i + 2) as *mut u8) = color.b;
                i += 3;
			}
		}
	}

	pub unsafe fn char_bitmap(&self, point: Point, bitmap_location: *const u8, color: Color){
        for row in 0..16 {
            let row_data = *((bitmap_location as u32 + row) as *const u8);
            for col in 0..8 {
                let pixel = (row_data >> (7 - col)) & 1;
                if pixel > 0 {
                    self.pixel(Point::new(point.x + col as i32, point.y + row as i32), color);
                }
            }
        }
	}

	pub fn char(&self, point: Point, character: char, color: Color){
        unsafe{
            if FONT_LOCATION > 0 {
                self.char_bitmap(point, (FONT_LOCATION + 16*(character as u32)) as *const u8, color);
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
        for i in 0..((*self.mode_info).xresolution as u32 - point.x as u32)/8 {
            let character = *((c_text as u32 + i) as *const u8);
            if character == 0 {
                break;
            }
            self.char(cursor, character as char, color);
            cursor.x += 8;
        }
    }

	pub fn window(&self, window: &Window){
		self.rect(Point::new(window.point.x - 2, window.point.y - 18), Size::new(window.size.width + 4, window.size.height + 16 + 4), Color::new(0, 0, 0));
		self.rect(Point::new(window.point.x, window.point.y), Size::new(window.size.width, window.size.height), Color::new(128, 128, 128));
		let mut cursor = Point::new(window.point.x, window.point.y - 16);
		for character in window.title.chars() {
			if cursor.x + 8 <= window.point.x + window.size.width as i32 {
				self.char(cursor, character, Color::new(255, 255, 255));
			}
			cursor.x += 8;
		}
	}

	pub fn copy(&self){
        unsafe{
            let mut i = 0;
            while i < (*self.mode_info).yresolution as u32 * (*self.mode_info).bytesperscanline as u32 {
                let data: u64 = *(((OFFSCREENLOCATION as u32) + i) as *const u64);
                *((((*self.mode_info).physbaseptr as u32) + i) as *mut u64) = data;
                i += 8;
            }
		}
	}
}
