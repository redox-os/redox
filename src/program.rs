#![feature(alloc)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(core_simd)]
#![feature(core_slice_ext)]
#![feature(core_str_ext)]
#![feature(fnbox)]
#![feature(fundamental)]
#![feature(lang_items)]
#![feature(no_std)]
#![feature(unboxed_closures)]
#![feature(unsafe_no_drop_flag)]
#![no_std]

extern crate alloc;

#[macro_use]
extern crate mopa;

use application::Application;

use core::mem::size_of;

use common::memory::*;

use programs::common::*;

#[path="APPLICATION_PATH"]
mod application;

mod common {
    pub mod debug;
    pub mod memory;
    pub mod pci;
    pub mod pio;
    pub mod random;
    pub mod resource;
    pub mod string;
    pub mod vec;
}

mod drivers {
    pub mod disk;
    pub mod keyboard;
    pub mod mouse;
}

mod filesystems {
    pub mod unfs;
}

mod graphics {
    pub mod bmp;
    pub mod color;
    pub mod display;
    pub mod point;
    pub mod size;
    pub mod window;
}

mod programs {
    pub mod common;
}

//Class wrappers

static mut application: *mut Application = 0 as *mut Application;

#[no_mangle]
pub unsafe fn entry(){
    application = alloc(size_of::<Application>()) as *mut Application;
    *application = Application::new();
}

#[no_mangle]
pub unsafe fn draw(display: &Display, events: &mut Vec<URL>) -> bool{
    if application as usize > 0 {
        return (*application).draw(display, events);
    }else{
        return false;
    }
}

#[no_mangle]
pub unsafe fn on_key(events: &mut Vec<URL>, key_event: KeyEvent){
    if application as usize > 0{
        (*application).on_key(events, key_event);
    }
}

#[no_mangle]
pub unsafe fn on_mouse(events: &mut Vec<URL>, mouse_point: Point, mouse_event: MouseEvent, allow_catch: bool) -> bool{
    if application as usize > 0 {
        return (*application).on_mouse(events, mouse_point, mouse_event, allow_catch);
    }else{
        return false;
    }
}

#[no_mangle]
pub extern "C" fn memmove(dst: *mut u8, src: *const u8, len: isize){
    unsafe {
        if src < dst {
            let mut i = len;
            while i > 0 {
                i -= 1;
                *dst.offset(i) = *src.offset(i);
            }
        }else{
            let mut i = 0;
            while i < len {
                *dst.offset(i) = *src.offset(i);
                i += 1;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn memcpy(dst: *mut u8, src: *const u8, len: isize){
    unsafe {
        let mut i = 0;
        while i < len {
            *dst.offset(i) = *src.offset(i);
            i += 1;
        }
    }
}

#[no_mangle]
pub extern "C" fn memset(src: *mut u8, c: i32, len: isize) {
    unsafe {
        let mut i = 0;
        while i < len {
            *src.offset(i) = c as u8;
            i += 1;
        }
    }
}
