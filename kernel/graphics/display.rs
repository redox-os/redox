use alloc::boxed::Box;

use core::cmp;

use arch::memory;

use super::FONT;
use super::color::Color;

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
            };

            Some(ret)
        } else {
            None
        }
    }

    // Optimized {
    pub unsafe fn copy_run(mut src: usize, mut dst: usize, len: usize) {
        let end = dst + len;
        while dst < end {
            *(src as *mut u32) = *(dst as *mut u32);
            src += 4;
            dst += 4;
        }
    }

    // Optimized {
    pub unsafe fn set_run(data: u32, mut dst: usize, len: usize) {
        let end = dst + len;
        while dst < end {
            *(dst as *mut u32) = data;
            dst += 4;
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
                Display::copy_run(self.offscreen, self.offscreen + offset, self.size - offset);
                Display::set_run(0, self.offscreen + self.size - offset, offset);
            }
        }
    }

    /// Flip the display
    pub fn flip(&self) {
        unsafe {
            Display::copy_run(self.onscreen, self.offscreen, self.size);
        }
    }

    /// Draw a rectangle
    pub fn rect(&self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        let data = color.data;

        let start_y = cmp::min(self.height - 1, y);
        let end_y = cmp::min(self.height - 1, y + h);

        let start_x = cmp::min(self.width - 1, x) * 4;
        let len = cmp::min(self.width - 1, x + w) * 4 - start_x;

        for y in start_y..end_y {
            unsafe {
                Display::set_run(data,
                                 self.offscreen + y * self.bytesperrow + start_x,
                                 len);
            }
        }
    }

    /// Draw a char
    pub fn char(&self, x: usize, y: usize, character: char, color: Color) {
        if x + 8 <= self.width && y + 16 <= self.height {
            let data = color.data;
            let mut dst = self.offscreen + y * self.bytesperrow + x * 4;

            let font_i = 16 * (character as usize);
            for row in 0..16 {
                let row_data = FONT[font_i + row];
                for col in 0..8 {
                    if (row_data >> (7 - col)) & 1 == 1 {
                        unsafe { *((dst + col * 4) as *mut u32) = data; }
                    }
                }
                dst += self.bytesperrow;
            }
        }
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe {
            if self.offscreen > 0 {
                memory::unalloc(self.offscreen);
            }
        }
    }
}
