use core::{cmp, slice};
use ransid::{Console, Event};
use spin::Mutex;

use memory::Frame;
use paging::{ActivePageTable, PhysicalAddress, entry};

#[cfg(target_arch = "x86_64")]
#[allow(unused_assignments)]
#[inline(always)]
#[cold]
unsafe fn fast_copy64(dst: *mut u64, src: *const u64, len: usize) {
    asm!("cld
        rep movsq"
        :
        : "{rdi}"(dst as usize), "{rsi}"(src as usize), "{rcx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}

#[cfg(target_arch = "x86_64")]
#[allow(unused_assignments)]
#[inline(always)]
#[cold]
unsafe fn fast_set(dst: *mut u32, src: u32, len: usize) {
    asm!("cld
        rep stosd"
        :
        : "{rdi}"(dst as usize), "{eax}"(src), "{rcx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}

#[cfg(target_arch = "x86_64")]
#[allow(unused_assignments)]
#[inline(always)]
#[cold]
unsafe fn fast_set64(dst: *mut u64, src: u64, len: usize) {
    asm!("cld
        rep stosq"
        :
        : "{rdi}"(dst as usize), "{rax}"(src), "{rcx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}

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

pub static CONSOLE: Mutex<Option<Console>> = Mutex::new(None);

pub static DISPLAY: Mutex<Option<Display>> = Mutex::new(None);

static FONT: &'static [u8] = include_bytes!("../../../../res/unifont.font");

pub unsafe fn init(active_table: &mut ActivePageTable) {
    active_table.identity_map(Frame::containing_address(PhysicalAddress::new(0x5200)), entry::PRESENT | entry::NO_EXECUTE);

    let mode_info = &*(0x5200 as *const VBEModeInfo);
    if mode_info.physbaseptr > 0 {
        let width = mode_info.xresolution as usize;
        let height = mode_info.yresolution as usize;
        let onscreen = mode_info.physbaseptr as usize;
        let size = width * height;

        {
            let start_frame = Frame::containing_address(PhysicalAddress::new(onscreen));
            let end_frame = Frame::containing_address(PhysicalAddress::new(onscreen + size * 4 - 1));
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                active_table.identity_map(frame, /*actually sets PAT for write combining*/ entry::HUGE_PAGE | entry::PRESENT | entry::WRITABLE | entry::NO_EXECUTE);
            }
        }

        fast_set64(onscreen as *mut u64, 0, size/2);

        let offscreen = ::allocator::__rust_allocate(size * 4, 8);
        fast_set64(offscreen as *mut u64, 0, size/2);

        *CONSOLE.lock() = Some(Console::new(width/8, height/16));
        *DISPLAY.lock() = Some(Display::new(width, height,
            slice::from_raw_parts_mut(onscreen as *mut u32, size),
            slice::from_raw_parts_mut(offscreen as *mut u32, size)
        ));
    }
}

pub unsafe fn init_ap(active_table: &mut ActivePageTable) {
    active_table.identity_map(Frame::containing_address(PhysicalAddress::new(0x5200)), entry::PRESENT | entry::NO_EXECUTE);

    let mode_info = &*(0x5200 as *const VBEModeInfo);
    if mode_info.physbaseptr > 0 {
        let width = mode_info.xresolution as usize;
        let height = mode_info.yresolution as usize;
        let start = mode_info.physbaseptr as usize;
        let size = width * height;

        {
            let start_frame = Frame::containing_address(PhysicalAddress::new(start));
            let end_frame = Frame::containing_address(PhysicalAddress::new(start + size * 4 - 1));
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                active_table.identity_map(frame, entry::PRESENT | entry::WRITABLE | entry::NO_EXECUTE);
            }
        }
    }
}

/// A display
pub struct Display {
    pub width: usize,
    pub height: usize,
    pub onscreen: &'static mut [u32],
    pub offscreen: &'static mut [u32],
}

impl Display {
    fn new(width: usize, height: usize, onscreen: &'static mut [u32], offscreen: &'static mut [u32]) -> Display {
        Display {
            width: width,
            height: height,
            onscreen: onscreen,
            offscreen: offscreen,
        }
    }

    /// Draw a rectangle
    fn rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        let start_y = cmp::min(self.height - 1, y);
        let end_y = cmp::min(self.height, y + h);

        let start_x = cmp::min(self.width - 1, x);
        let len = cmp::min(self.width, x + w) - start_x;

        let mut offscreen_ptr = self.offscreen.as_mut_ptr() as usize;
        let mut onscreen_ptr = self.onscreen.as_mut_ptr() as usize;

        let stride = self.width * 4;

        let offset = y * stride + start_x * 4;
        offscreen_ptr += offset;
        onscreen_ptr += offset;

        let mut rows = end_y - start_y;
        while rows > 0 {
            unsafe {
                fast_set(offscreen_ptr as *mut u32, color, len);
                fast_set(onscreen_ptr as *mut u32, color, len);
            }
            offscreen_ptr += stride;
            onscreen_ptr += stride;
            rows -= 1;
        }
    }

    /// Draw a character
    fn char(&mut self, x: usize, y: usize, character: char, color: u32) {
        if x + 8 <= self.width && y + 16 <= self.height {
            let mut font_i = 16 * (character as usize);
            let font_end = font_i + 16;
            if font_end <= FONT.len() {
                let mut offscreen_ptr = self.offscreen.as_mut_ptr() as usize;
                let mut onscreen_ptr = self.onscreen.as_mut_ptr() as usize;

                let stride = self.width * 4;

                let offset = y * stride + x * 4;
                offscreen_ptr += offset;
                onscreen_ptr += offset;

                while font_i < font_end {
                    let mut row_data = FONT[font_i];
                    let mut col = 8;
                    while col > 0 {
                        col -= 1;
                        if row_data & 1 == 1 {
                            unsafe {
                                *((offscreen_ptr + col * 4) as *mut u32) = color;
                            }
                        }
                        row_data = row_data >> 1;
                    }

                    unsafe {
                        fast_copy64(onscreen_ptr as *mut u64, offscreen_ptr as *const u64, 4);
                    }

                    offscreen_ptr += stride;
                    onscreen_ptr += stride;
                    font_i += 1;
                }
            }
        }
    }

    /// Scroll display
    pub fn scroll(&mut self, rows: usize, color: u32) {
        let data = (color as u64) << 32 | color as u64;

        let width = self.width/2;
        let height = self.height;
        if rows > 0 && rows < height {
            let off1 = rows * width;
            let off2 = height * width - off1;
            unsafe {
                let data_ptr = self.offscreen.as_mut_ptr() as *mut u64;
                fast_copy64(data_ptr, data_ptr.offset(off1 as isize), off2);
                fast_set64(data_ptr.offset(off2 as isize), data, off1);

                fast_copy64(self.onscreen.as_mut_ptr() as *mut u64, data_ptr, off1 + off2);
            }
        }
    }

    /// Handle ransid event
    pub fn event(&mut self, event: Event) {
        match event {
            Event::Char { x, y, c, color, .. } => {
                self.char(x * 8, y * 16, c, color.data);
            },
            Event::Rect { x, y, w, h, color } => {
                self.rect(x * 8, y * 16, w * 8, h * 16, color.data);
            },
            Event::Scroll { rows, color } => {
                self.scroll(rows * 16, color.data);
            }
        }
    }
}
