use core::cmp::min;
use core::cmp::max;
use core::mem::size_of;
use core::simd::*;

use common::memory::*;
use common::string::*;

use drivers::disk::*;

use filesystems::unfs::*;

use graphics::bmp::*;
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
    offscreen: usize,
    onscreen: usize,
    size: usize,
    fonts: usize,
    background: BMP,
    cursor: BMP,
    bytesperrow: usize,
    pub width: usize,
    pub height: usize
}

impl Display {
    pub fn new() -> Display {
        unsafe{
            let mode_info = &*(VBEMODEINFOLOCATION as *const VBEModeInfo);

            let unfs = UnFS::new(Disk::new());

            let fonts = unfs.load(&String::from_str("unifont.font"));

            let background_data = unfs.load(&String::from_str("background.bmp"));
            let background = BMP::from_data(background_data);
            unalloc(background_data);

            let cursor_data = unfs.load(&String::from_str("cursor.bmp"));
            let cursor = BMP::from_data(cursor_data);
            unalloc(cursor_data);

            Display {
                offscreen: alloc(mode_info.bytesperscanline as usize * mode_info.yresolution as usize),
                onscreen: mode_info.physbaseptr as usize,
                size: mode_info.bytesperscanline as usize * mode_info.yresolution as usize,
                fonts: fonts,
                background: background,
                cursor: cursor,
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
        let data = color.data;
        let alpha = (color.data & 0xFF000000) >> 24;

        if alpha > 0 {
            let start_y = max(0, min(self.height as isize - 1, point.y)) as usize;
            let end_y = max(0, min(self.height as isize - 1, point.y + size.height as isize)) as usize;

            let start_x = max(0, min(self.width as isize - 1, point.x)) as usize * 4;
            let len = max(0, min(self.width as isize - 1, point.x + size.width as isize)) as usize * 4 - start_x;

            if alpha >= 255 {
                for y in start_y..end_y {
                    unsafe{
                        Display::set_run(data, self.offscreen + y * self.bytesperrow + start_x, len);
                    }
                }
            }else{
                let n_alpha = 255 - alpha;
                let r = (((data >> 16) & 0xFF) * alpha) >> 8;
                let g = (((data >> 8) & 0xFF) * alpha) >> 8;
                let b = ((data & 0xFF) * alpha) >> 8;
                let premul = (r << 16) | (g << 8) | b;
                for y in start_y..end_y {
                    unsafe{
                        Display::set_run_alpha(premul, n_alpha, self.offscreen + y * self.bytesperrow + start_x, len);
                    }
                }
            }
        }
    }

    pub fn image(&self, point: Point, data: usize, size: Size){
        let start_y = max(0, point.y) as usize;
        let end_y = min(self.height as isize, point.y + size.height as isize) as usize;

        let start_x = max(0, point.x) as usize;
        let len = min(self.width as isize, point.x + size.width as isize) as usize * 4 - start_x * 4;
        let offscreen_offset = self.offscreen + start_x * 4;

        let bytesperrow = size.width * 4;
        let data_offset = data - start_y * bytesperrow - (point.x - start_x as isize) as usize * 4;

        for y in start_y..end_y{
            unsafe{
                Display::copy_run(data_offset + y * bytesperrow, offscreen_offset + y * self.bytesperrow, len);
            }
        }
    }
    /* } Optimized */


    pub fn image_alpha(&self, point: Point, data: usize, size: Size){
        let start_y = max(0, point.y) as usize;
        let end_y = min(self.height as isize, point.y + size.height as isize) as usize;

        let start_x = max(0, point.x) as usize;
        let len = min(self.width as isize, point.x + size.width as isize) as usize * 4 - start_x * 4;
        let offscreen_offset = self.offscreen + start_x * 4;

        let bytesperrow = size.width * 4;
        let data_offset = data - start_y * bytesperrow - (point.x - start_x as isize) as usize * 4;

        for y in start_y..end_y{
            unsafe{
                Display::copy_run_alpha(data_offset + y * bytesperrow, offscreen_offset + y * self.bytesperrow, len);
            }
        }
    }

    pub fn background(&self){
        if self.background.data > 0 {
            self.image(Point::new(0, 0), self.background.data, self.background.size);
        }else{
            self.set(Color::new(64, 64, 64));
        }
    }

    pub fn cursor(&self, point: Point){
        if self.cursor.data > 0 {
            self.image_alpha(point, self.cursor.data, self.cursor.size);
        }else{
            self.char(Point::new(point.x - 3, point.y - 9), 'X', Color::new(255, 255, 255));
        }
    }

    //TODO: SIMD to optimize
    pub unsafe fn set_run_alpha(premul: u32, n_alpha: u32, dst: usize, len: usize){
        let mut i = 0;
        while len - i >= size_of::<u32>() {
            let orig = *((dst + i) as *const u32);
            let r = (((orig >> 16) & 0xFF) * n_alpha) >> 8;
            let g = (((orig >> 8) & 0xFF) * n_alpha) >> 8;
            let b = ((orig & 0xFF) * n_alpha) >> 8;
            *((dst + i) as *mut u32) = ((r << 16) | (g << 8) | b) + premul;
            i += size_of::<u32>();
        }
    }

    //TODO: SIMD to optimize
    pub unsafe fn copy_run_alpha(src: usize, dst: usize, len: usize){
        let mut i = 0;
        while len - i >= size_of::<u32>() {
            let new = *((src + i) as *const u32);
            let alpha = (new >> 24) & 0xFF;
            if alpha > 0 {
                if alpha >= 255 {
                    *((dst + i) as *mut u32) = new;
                }else{
                    let n_r = (((new >> 16) & 0xFF) * alpha) >> 8;
                    let n_g = (((new >> 8) & 0xFF) * alpha) >> 8;
                    let n_b = ((new & 0xFF) * alpha) >> 8;

                    let orig = *((dst + i) as *const u32);
                    let n_alpha = 255 - alpha;
                    let o_r = (((orig >> 16) & 0xFF) * n_alpha) >> 8;
                    let o_g = (((orig >> 8) & 0xFF) * n_alpha) >> 8;
                    let o_b = ((orig & 0xFF) * n_alpha) >> 8;

                    *((dst + i) as *mut u32) = ((o_r << 16) | (o_g << 8) | o_b) + ((n_r << 16) | (n_g << 8) | n_b);
                }
            }
            i += size_of::<u32>();
        }
    }

    pub fn pixel(&self, point: Point, color: Color){
        unsafe{
            if point.x >= 0 && point.x < self.width as isize && point.y >= 0 && point.y < self.height as isize {
                *((self.offscreen + point.y as usize * self.bytesperrow + point.x as usize * 4) as *mut u32) = color.data;
            }
        }
    }

    pub fn char(&self, point: Point, character: char, color: Color){
        unsafe{
            if self.fonts > 0 {
                let bitmap_location = self.fonts + 16*(character as usize);
                for row in 0..16 {
                    let row_data = *((bitmap_location + row) as *const u8);
                    for col in 0..8 {
                        let pixel = (row_data >> (7 - col)) & 1;
                        if pixel > 0 {
                            self.pixel(Point::new(point.x + col, point.y + row as isize), color);
                        }
                    }
                }
            }
        }
    }

    pub fn text(&self, point: Point, text: &String, color: Color){
        let mut cursor = Point::new(point.x, point.y);
        for c in text.iter() {
            self.char(cursor, c, color);
            cursor.x += 8;
        }
    }
}
