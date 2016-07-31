use alloc::boxed::Box;

use core::cmp;

use arch::memory;

use system::graphics::{fast_copy, fast_set};

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
    pub offscreen: *mut u32,
    pub onscreen: *mut u32,
    pub size: usize,
    pub width: usize,
    pub height: usize,
}

impl Display {
    pub fn root() -> Option<Box<Self>> {
        if let Some(mode_info) = unsafe { VBEMODEINFO } {
            let ret = box Display {
                offscreen: unsafe { memory::alloc(mode_info.xresolution as usize *
                                         mode_info.yresolution as usize * 4) as *mut u32 },
                onscreen: mode_info.physbaseptr as usize as *mut u32,
                size: mode_info.xresolution as usize * mode_info.yresolution as usize,
                width: mode_info.xresolution as usize,
                height: mode_info.yresolution as usize,
            };

            ret.set(Color::new(0, 0, 0));

            Some(ret)
        } else {
            None
        }
    }

    /// Set the color
    pub fn set(&self, color: Color) {
        unsafe {
            fast_set(self.offscreen, color.data, self.size);
        }
    }

    /// Scroll the display
    pub fn scroll(&self, rows: usize, color: Color) {
        if rows > 0 && rows < self.height {
            let offset = rows * self.width;
            unsafe {
                fast_copy(self.offscreen, self.offscreen.offset(offset as isize), self.size - offset);
                fast_set(self.offscreen.offset((self.size - offset) as isize), color.data, offset);
            }
        }
    }

    /// Flip the display
    pub fn flip(&self) {
        unsafe {
            fast_copy(self.onscreen, self.offscreen, self.size);
        }
    }

    /// Draw a rectangle
    pub fn rect(&self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        let data = color.data;

        let start_y = cmp::min(self.height - 1, y);
        let end_y = cmp::min(self.height - 1, y + h);

        let start_x = cmp::min(self.width - 1, x);
        let len = cmp::min(self.width - 1, x + w) - start_x;

        for y in start_y..end_y {
            unsafe {
                fast_set(self.offscreen.offset((y * self.width + start_x) as isize), data, len);
            }
        }
    }

    /// Draw a char
    pub fn char(&self, x: usize, y: usize, character: char, color: Color) {
        if x + 8 <= self.width && y + 16 <= self.height {
            let data = color.data;
            let mut dst = unsafe { self.offscreen.offset((y * self.width + x) as isize) };

            let font_i = 16 * (character as usize);
            if font_i + 16 <= FONT.len() {
                for row in 0..16 {
                    let row_data = FONT[font_i + row];
                    for col in 0..8 {
                        if (row_data >> (7 - col)) & 1 == 1 {
                            unsafe { *dst.offset(col) = data; }
                        }
                    }
                    dst = unsafe { dst.offset(self.width as isize) };
                }
            }
        }
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe {
            if self.offscreen as usize > 0 {
                memory::unalloc(self.offscreen as usize);
            }
        }
    }
}
