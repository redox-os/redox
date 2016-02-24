use alloc::boxed::Box;

use core::{cmp, mem};
// use core::simd::*;

use arch::memory;

use super::FONT;
use super::color::Color;
use super::point::Point;
use super::size::Size;

/// The info of the VBE mode
#[derive(Copy, Clone, Default, Debug)]
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
    pub xresolution: u16,
    pub yresolution: u16,
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

pub static mut VBEMODEINFO: Option<VBEModeInfo> = None;

pub unsafe fn vbe_init(){
    let mode_info = *(0x5200 as *const VBEModeInfo);
    if mode_info.physbaseptr > 0 {
        VBEMODEINFO = Some(mode_info);
    }else{
        VBEMODEINFO = None;
    }
}

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
    pub fn root() -> Option<Box<Self>> {
        if let Some(mode_info) = unsafe { VBEMODEINFO } {
            let ret = box Display {
                offscreen: unsafe { memory::alloc(mode_info.bytesperscanline as usize *
                                         mode_info.yresolution as usize) },
                onscreen: mode_info.physbaseptr as usize,
                size: mode_info.bytesperscanline as usize * mode_info.yresolution as usize,
                bytesperrow: mode_info.bytesperscanline as usize,
                width: mode_info.xresolution as usize,
                height: mode_info.yresolution as usize,
                root: true,
            };

            Some(ret)
        } else {
            None
        }
    }

    // Optimized {
    pub unsafe fn set_run(data: u32, dst: usize, len: usize) {
        let mut i = 0;
        // Only use 16 byte transfer if possible
        // if len - (dst + i) % 16 >= mem::size_of::<u32x4>() {
        // Align 16
        // while (dst + i) % 16 != 0 && len - i >= mem::size_of::<u32>() {
        // ((dst + i) as *mut u32) = data;
        // i += mem::size_of::<u32>();
        // }
        // While 16 byte transfers
        // let simd: u32x4 = u32x4(data, data, data, data);
        // while len - i >= mem::size_of::<u32x4>() {
        // ((dst + i) as *mut u32x4) = simd;
        // i += mem::size_of::<u32x4>();
        // }
        // }
        //
        // Everything after last 16 byte transfer
        while len - i >= mem::size_of::<u32>() {
            *((dst + i) as *mut u32) = data;
            i += mem::size_of::<u32>();
        }
    }

    pub unsafe fn copy_run(src: usize, dst: usize, len: usize) {
        let mut i = 0;
        // Only use 16 byte transfer if possible
        // if (src + i) % 16 == (dst + i) % 16 {
        // Align 16
        // while (dst + i) % 16 != 0 && len - i >= mem::size_of::<u32>() {
        // ((dst + i) as *mut u32) = *((src + i) as *const u32);
        // i += mem::size_of::<u32>();
        // }
        // While 16 byte transfers
        // while len - i >= mem::size_of::<u32x4>() {
        // ((dst + i) as *mut u32x4) = *((src + i) as *const u32x4);
        // i += mem::size_of::<u32x4>();
        // }
        // }
        //
        // Everything after last 16 byte transfer
        while len - i >= mem::size_of::<u32>() {
            *((dst + i) as *mut u32) = *((src + i) as *const u32);
            i += mem::size_of::<u32>();
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
                Display::copy_run(self.offscreen + offset, self.offscreen, self.size - offset);
                Display::set_run(0, self.offscreen + self.size - offset, offset);
            }
        }
    }

    /// Flip the display
    pub fn flip(&self) {
        unsafe {
            if self.root {
                Display::copy_run(self.offscreen, self.onscreen, self.size);
            } else {
                let self_mut: *mut Self = mem::transmute(self);
                mem::swap(&mut (*self_mut).offscreen, &mut (*self_mut).onscreen);
            }
        }
    }

    /// Draw a rectangle
    pub fn rect(&self, point: Point, size: Size, color: Color) {
        let data = color.data;

        let start_y = cmp::max(0, cmp::min(self.height as isize - 1, point.y)) as usize;
        let end_y = cmp::max(0,
                             cmp::min(self.height as isize - 1,
                                      point.y +
                                      size.height as isize)) as usize;

        let start_x = cmp::max(0, cmp::min(self.width as isize - 1, point.x)) as usize * 4;
        let len = cmp::max(0,
                           cmp::min(self.width as isize - 1,
                                    point.x +
                                    size.width as isize)) as usize * 4 -
                  start_x;

        for y in start_y..end_y {
            unsafe {
                Display::set_run(data,
                                 self.offscreen + y * self.bytesperrow + start_x,
                                 len);
            }
        }
    }

    /// Set the color of a pixel
    pub fn pixel(&self, point: Point, color: Color) {
        unsafe {
            if point.x >= 0 && point.x < self.width as isize && point.y >= 0 &&
               point.y < self.height as isize {
                *((self.offscreen + point.y as usize * self.bytesperrow +
                   point.x as usize * 4) as *mut u32) = color.data;
            }
        }
    }

    /// Draw a char
    pub fn char(&self, point: Point, character: char, color: Color) {
        let font_i = 16 * (character as usize);
        for row in 0..16 {
            let row_data = FONT[font_i + row];
            for col in 0..8 {
                let pixel = (row_data >> (7 - col)) & 1;
                if pixel > 0 {
                    self.pixel(Point::new(point.x + col, point.y + row as isize), color);
                }
            }
        }
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe {
            if self.offscreen > 0 {
                memory::unalloc(self.offscreen);
                self.offscreen = 0;
            }
            if !self.root && self.onscreen > 0 {
                memory::unalloc(self.onscreen);
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
