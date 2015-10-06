use core::cmp::min;
use core::cmp::max;
use core::mem::size_of;
use core::mem::swap;
use core::mem::transmute;
use core::ops::Drop;
use core::simd::*;

use common::scheduler::*;

use graphics::color::*;
use graphics::point::*;
use graphics::size::*;

use syscall::call::*;

/// The info of the VBE mode
#[derive(Copy, Clone)]
#[repr(packed)]
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
    offscreenmemsize: u16,
}

const VBEMODEINFO: *const VBEModeInfo = 0x5200 as *const VBEModeInfo;

pub const FONTS: *mut usize = 0x200008 as *mut usize;

/// A display
pub struct Display {
    pub offscreen: usize,
    pub onscreen: usize,
    pub size: usize,
    pub bytesperrow: usize,
    pub width: usize,
    pub height: usize,
    pub root: bool,
}

impl Display {
    pub unsafe fn root() -> Self {
        let mode_info = &*VBEMODEINFO;

        let ret = Display {
            offscreen: sys_alloc(mode_info.bytesperscanline as usize *
                                 mode_info.yresolution as usize),
            onscreen: mode_info.physbaseptr as usize,
            size: mode_info.bytesperscanline as usize * mode_info.yresolution as usize,
            bytesperrow: mode_info.bytesperscanline as usize,
            width: mode_info.xresolution as usize,
            height: mode_info.yresolution as usize,
            root: true,
        };

        ret.set(Color::new(0, 0, 0));
        ret.flip();

        ret
    }

    /// Create a new display
    pub fn new(width: usize, height: usize) -> Self {
        unsafe {
            let bytesperrow = width * 4;
            let memory_size = bytesperrow * height;

            let ret = Display {
                offscreen: sys_alloc(memory_size),
                onscreen: sys_alloc(memory_size),
                size: memory_size,
                bytesperrow: bytesperrow,
                width: width,
                height: height,
                root: false,
            };

            ret.set(Color::new(0, 0, 0));
            ret.flip();

            ret
        }
    }

    /* Optimized { */
    pub unsafe fn set_run(data: u32, dst: usize, len: usize) {
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

    pub unsafe fn copy_run(src: usize, dst: usize, len: usize) {
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

    /// Set the color
    pub fn set(&self, color: Color) {
        unsafe {
            Display::set_run(color.data, self.offscreen, self.size);
        }
    }

    /// Scroll the display
    pub fn scroll(&self, rows: usize) {
        if rows > 0 && rows < self.height {
            let offset = rows * self.bytesperrow;
            unsafe {
                Display::copy_run(self.offscreen + offset,
                                  self.offscreen,
                                  self.size - offset);
                Display::set_run(0, self.offscreen + self.size - offset, offset);
            }
        }
    }

    /// Flip the display
    pub fn flip(&self) {
        unsafe {
            let reenable = start_no_ints();
            if self.root {
                Display::copy_run(self.offscreen, self.onscreen, self.size);
            } else {
                let self_mut: *mut Self = transmute(self);
                swap(&mut (*self_mut).offscreen,
                     &mut (*self_mut).onscreen);
            }
            end_no_ints(reenable);
        }
    }

    /// Draw a rectangle
    pub fn rect(&self, point: Point, size: Size, color: Color) {
        let data = color.data;
        let alpha = (color.data & 0xFF000000) >> 24;

        if alpha > 0 {
            let start_y = max(0, min(self.height as isize - 1, point.y)) as usize;
            let end_y =
                max(0, min(self.height as isize - 1, point.y + size.height as isize)) as usize;

            let start_x = max(0, min(self.width as isize - 1, point.x)) as usize * 4;
            let len = max(0, min(self.width as isize - 1, point.x + size.width as isize)) as usize *
                      4 - start_x;

            if alpha >= 255 {
                for y in start_y..end_y {
                    unsafe {
                        Display::set_run(data,
                                         self.offscreen + y * self.bytesperrow + start_x,
                                         len);
                    }
                }
            } else {
                let n_alpha = 255 - alpha;
                let r = (((data >> 16) & 0xFF) * alpha) >> 8;
                let g = (((data >> 8) & 0xFF) * alpha) >> 8;
                let b = ((data & 0xFF) * alpha) >> 8;
                let premul = (r << 16) | (g << 8) | b;
                for y in start_y..end_y {
                    unsafe {
                        Display::set_run_alpha(premul,
                                               n_alpha,
                                               self.offscreen + y * self.bytesperrow + start_x,
                                               len);
                    }
                }
            }
        }
    }

    /// Draw an image
    pub unsafe fn image(&self, point: Point, data: *const u32, size: Size) {
        let start_y = max(0, point.y) as usize;
        let end_y = min(self.height as isize, point.y + size.height as isize) as usize;

        let start_x = max(0, point.x) as usize;
        let len = min(self.width as isize, point.x + size.width as isize) as usize * 4 -
                  start_x * 4;
        let offscreen_offset = self.offscreen + start_x * 4;

        let bytesperrow = size.width * 4;
        let data_offset = data as usize - start_y * bytesperrow -
                          (point.x - start_x as isize) as usize * 4;

        for y in start_y..end_y {
            Display::copy_run(data_offset + y * bytesperrow,
                              offscreen_offset + y * self.bytesperrow,
                              len);
        }
    }
    /* } Optimized */

    /// Draw a image with opacity
    pub unsafe fn image_alpha(&self, point: Point, data: *const u32, size: Size) {
        let start_y = max(0, point.y) as usize;
        let end_y = min(self.height as isize, point.y + size.height as isize) as usize;

        let start_x = max(0, point.x) as usize;
        let len = min(self.width as isize, point.x + size.width as isize) as usize * 4 -
                  start_x * 4;
        let offscreen_offset = self.offscreen + start_x * 4;

        let bytesperrow = size.width * 4;
        let data_offset = data as usize - start_y * bytesperrow -
                          (point.x - start_x as isize) as usize * 4;

        for y in start_y..end_y {
            Display::copy_run_alpha(data_offset + y * bytesperrow,
                                    offscreen_offset + y * self.bytesperrow,
                                    len);
        }
    }

    //TODO: SIMD to optimize
    pub unsafe fn set_run_alpha(premul: u32, n_alpha: u32, dst: usize, len: usize) {
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
    pub unsafe fn copy_run_alpha(src: usize, dst: usize, len: usize) {
        let mut i = 0;
        while len - i >= size_of::<u32>() {
            let new = *((src + i) as *const u32);
            let alpha = (new >> 24) & 0xFF;
            if alpha > 0 {
                if alpha >= 255 {
                    *((dst + i) as *mut u32) = new;
                } else {
                    let n_r = (((new >> 16) & 0xFF) * alpha) >> 8;
                    let n_g = (((new >> 8) & 0xFF) * alpha) >> 8;
                    let n_b = ((new & 0xFF) * alpha) >> 8;

                    let orig = *((dst + i) as *const u32);
                    let n_alpha = 255 - alpha;
                    let o_r = (((orig >> 16) & 0xFF) * n_alpha) >> 8;
                    let o_g = (((orig >> 8) & 0xFF) * n_alpha) >> 8;
                    let o_b = ((orig & 0xFF) * n_alpha) >> 8;

                    *((dst + i) as *mut u32) = ((o_r << 16) | (o_g << 8) | o_b) +
                                               ((n_r << 16) | (n_g << 8) | n_b);
                }
            }
            i += size_of::<u32>();
        }
    }

    /// Set the color of a pixel
    pub fn pixel(&self, point: Point, color: Color) {
        unsafe {
            if point.x >= 0 && point.x < self.width as isize && point.y >= 0 &&
               point.y < self.height as isize {
                *((self.offscreen + point.y as usize * self.bytesperrow + point.x as usize * 4) as *mut u32) = color.data;
            }
        }
    }

    /// Draw a char
    pub fn char(&self, point: Point, character: char, color: Color) {
        unsafe {
            if *FONTS > 0 {
                let bitmap_location = *FONTS + 16 * (character as usize);
                for row in 0..16 {
                    let row_data = *((bitmap_location + row) as *const u8);
                    for col in 0..8 {
                        let pixel = (row_data >> (7 - col)) & 1;
                        if pixel > 0 {
                            self.pixel(Point::new(point.x + col, point.y + row as isize),
                                       color);
                        }
                    }
                }
            }
        }
    }

    /* Cursor hacks { */
    pub unsafe fn image_alpha_onscreen(&self, point: Point, data: *const u32, size: Size) {
        let start_y = max(0, point.y) as usize;
        let end_y = min(self.height as isize, point.y + size.height as isize) as usize;

        let start_x = max(0, point.x) as usize;
        let len = min(self.width as isize, point.x + size.width as isize) as usize * 4 -
                  start_x * 4;
        let onscreen_offset = self.onscreen + start_x * 4;

        let bytesperrow = size.width * 4;
        let data_offset = data as usize - start_y * bytesperrow -
                          (point.x - start_x as isize) as usize * 4;

        for y in start_y..end_y {
            Display::copy_run_alpha(data_offset + y * bytesperrow,
                                    onscreen_offset + y * self.bytesperrow,
                                    len);
        }
    }

    /// Draw a pixel on the screen (absolute)
    pub fn pixel_onscreen(&self, point: Point, color: Color) {
        unsafe {
            if point.x >= 0 && point.x < self.width as isize && point.y >= 0 &&
               point.y < self.height as isize {
                *((self.onscreen + point.y as usize * self.bytesperrow + point.x as usize * 4) as *mut u32) = color.data;
            }
        }
    }

    /// Draw a char on the screen (absolute)
    pub fn char_onscreen(&self, point: Point, character: char, color: Color) {
        unsafe {
            if *FONTS > 0 {
                let bitmap_location = *FONTS + 16 * (character as usize);
                for row in 0..16 {
                    let row_data = *((bitmap_location + row) as *const u8);
                    for col in 0..8 {
                        let pixel = (row_data >> (7 - col)) & 1;
                        if pixel > 0 {
                            self.pixel_onscreen(Point::new(point.x + col, point.y + row as isize),
                                                color);
                        }
                    }
                }
            }
        }
    }
    /* } Cursor hacks */
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe {
            if self.offscreen > 0 {
                sys_unalloc(self.offscreen);
                self.offscreen = 0;
            }
            if !self.root && self.onscreen > 0 {
                sys_unalloc(self.onscreen);
                self.onscreen = 0;
            }
            self.size = 0;
            self.bytesperrow = 0;
            self.width = 0;
            self.height = 0;
            self.root = false;
        }
    }
}
