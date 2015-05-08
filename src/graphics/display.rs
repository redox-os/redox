use core::cmp::min;
use core::cmp::max;
use core::mem::size_of;
use core::simd::*;

use common::memory::*;
use common::string::*;

use drivers::disk::*;

use filesystems::unfs::*;

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

pub struct Display {
    mode_info: VBEModeInfo,
    offscreen: usize,
    onscreen: usize,
    size: usize,
    fonts: usize,
    bytesperrow: usize,
    pub width: usize,
    pub height: usize
}

impl Display {
    pub fn new() -> Display {
        unsafe{
            let mode_info = *(VBEMODEINFOLOCATION as *const VBEModeInfo);

            let fonts = UnFS::new(Disk::new()).load(&String::from_str("unifont.font"));

            Display {
                mode_info: mode_info,
                offscreen: alloc(mode_info.bytesperscanline as usize * mode_info.yresolution as usize),
                onscreen: mode_info.physbaseptr as usize,
                size: mode_info.bytesperscanline as usize * mode_info.yresolution as usize,
                fonts: fonts,
                bytesperrow: mode_info.bytesperscanline as usize,
                width: mode_info.xresolution as usize,
                height: mode_info.yresolution as usize
            }
        }
    }

    /* Optimized { */
    pub unsafe fn set_run(data: u32, dst: usize, len: usize){
        let mut i = 0;
        //Only use 16 byte transfer if possible
        if len - (dst + i) % 16 >= size_of::<u32x4>() {
            //Align 16
            while (dst + i) % 16 != 0 && len - i >= size_of::<u32>() {
                *((dst + i) as *mut u32) = data;
                i += size_of::<u32>();
            }
            //While 16 byte transfers
            let simd: u32x4 = u32x4(data, data, data, data);
            while len - i >= size_of::<u32x4>() {
                *((dst + i) as *mut u32x4) = simd;
                i += size_of::<u32x4>();
            }
        }
        //Everything after last 16 byte transfer
        while len - i >= size_of::<u32>() {
            *((dst + i) as *mut u32) = data;
            i += size_of::<u32>();
        }
    }

    pub unsafe fn copy_run(src: usize, dst: usize, len: usize){
        let mut i = 0;
        //Only use 16 byte transfer if possible
        if (src + i) % 16 == (dst + i) % 16 {
            //Align 16
            while (dst + i) % 16 != 0 && len - i >= size_of::<u32>() {
                *((dst + i) as *mut u32) = *((src + i) as *const u32);
                i += size_of::<u32>();
            }
            //While 16 byte transfers
            while len - i >= size_of::<u32x4>() {
                *((dst + i) as *mut u32x4) = *((src + i) as *const u32x4);
                i += size_of::<u32x4>();
            }
        }
        //Everything after last 16 byte transfer
        while len - i >= size_of::<u32>() {
            *((dst + i) as *mut u32) = *((src + i) as *const u32);
            i += size_of::<u32>();
        }
    }

    pub fn set(&self, color:Color){
        unsafe {
            Display::set_run(color.data, self.offscreen, self.size);
        }
    }

    pub fn flip(&self){
        unsafe{
            Display::copy_run(self.offscreen, self.onscreen, self.size);
        }
    }

    pub fn rect(&self, point: Point, size: Size, color: Color){
        let start_y = max(0, min(self.height as isize - 1, point.y)) as usize;
        let end_y = max(0, min(self.height as isize - 1, point.y + size.height as isize)) as usize;

        let start_x = max(0, min(self.width as isize - 1, point.x)) as usize * 4;
        let len = max(0, min(self.width as isize - 1, point.x + size.width as isize)) as usize * 4 - start_x;

        for y in start_y..end_y {
            unsafe{
                Display::set_run(color.data, self.offscreen + y * self.bytesperrow + start_x, len);
            }
        }
    }

    pub fn image(&self, point: Point, data: usize, size: Size){
        let start_y = max(0, point.y) as usize;
        let end_y = min(self.height as isize, point.y + size.height as isize) as usize;

        let start_x = max(0, point.x) as usize * 4;
        let len = min(self.width as isize, point.x + size.width as isize) as usize * 4 - start_x;

        let bytesperrow = size.width * 4;
        let data_offset = data - start_y * bytesperrow - (point.x - max(0, point.x)) as usize * 4;

        for y in start_y..end_y{
            unsafe{
                Display::copy_run(data_offset + y * bytesperrow, self.offscreen + y * self.bytesperrow + start_x, len);
            }
        }
    }
    /* } Optimized */

    pub fn pixel(&self, point: Point, color: Color){
        unsafe{
            if point.x >= 0 && point.x < self.width as isize && point.y >= 0 && point.y < self.height as isize {
                *((self.offscreen + point.y as usize * self.bytesperrow + point.x as usize * 4) as *mut u32) = color.data;
            }
        }
    }

    pub unsafe fn char_bitmap(&self, point: Point, bitmap_location: *const u8, color: Color){
        for row in 0..16 {
            let row_data = *((bitmap_location as usize + row) as *const u8);
            for col in 0..8 {
                let pixel = (row_data >> (7 - col)) & 1;
                if pixel > 0 {
                    self.pixel(Point::new(point.x + col, point.y + row as isize), color);
                }
            }
        }
    }

    pub fn char(&self, point: Point, character: char, color: Color){
        unsafe{
            if self.fonts > 0 {
                self.char_bitmap(point, (self.fonts + 16*(character as usize)) as *const u8, color);
            }
        }
    }

    pub fn text(&self, point: Point, text: &String, color: Color){
        let mut cursor = Point::new(point.x, point.y);
        for character in text.as_slice() {
            self.char(cursor, *character, color);
            cursor.x += 8;
        }
    }

    pub unsafe fn c_text(&self, point: Point, c_text: *const u8, color: Color){
        let mut cursor = Point::new(point.x, point.y);
        for i in 0..(self.width - point.x as usize)/8 {
            let character = *((c_text as usize + i) as *const u8);
            if character == 0 {
                break;
            }
            self.char(cursor, character as char, color);
            cursor.x += 8;
        }
    }
}
