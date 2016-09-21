#![feature(alloc)]
#![feature(asm)]
#![feature(heap_api)]

extern crate alloc;
extern crate syscall;

use std::fs::File;
use std::{slice, thread};
use syscall::{physmap, physunmap, MAP_WRITE, MAP_WRITE_COMBINE};

use display::Display;
use mode_info::VBEModeInfo;
use primitive::fast_set64;

pub mod display;
pub mod mode_info;
pub mod primitive;

fn main() {
    let width;
    let height;
    let physbaseptr;

    {
        let mode_info = unsafe { &*(physmap(0x5200, 4096, 0).expect("vesad: failed to map VBE info") as *const VBEModeInfo) };

        width = mode_info.xresolution as usize;
        height = mode_info.yresolution as usize;
        physbaseptr = mode_info.physbaseptr as usize;

        unsafe { let _ = physunmap(mode_info as *const _ as usize); }
    }

    if physbaseptr > 0 {
        thread::spawn(move || {
            let mut socket = File::create(":display").expect("vesad: failed to create display scheme");
            
            let size = width * height;

            let onscreen = unsafe { physmap(physbaseptr as usize, size * 4, MAP_WRITE | MAP_WRITE_COMBINE).expect("vesad: failed to map VBE LFB") };
            unsafe { fast_set64(onscreen as *mut u64, 0, size/2) };

            let offscreen = unsafe { alloc::heap::allocate(size * 4, 4096) };
            unsafe { fast_set64(offscreen as *mut u64, 0, size/2) };

            let mut display = Display::new(width, height,
                unsafe { slice::from_raw_parts_mut(onscreen as *mut u32, size) },
                unsafe { slice::from_raw_parts_mut(offscreen as *mut u32, size) }
            );

            display.rect(100, 100, 100, 100, 0xFF0000);
        });
    }
}
