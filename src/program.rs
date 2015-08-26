#![feature(alloc)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(core_simd)]
#![feature(core_slice_ext)]
#![feature(core_str_ext)]
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

use common::memory::*;

use programs::common::*;

#[path="APPLICATION_PATH"]
mod application;

mod common {
    pub mod debug;
    pub mod event;
    pub mod queue;
    pub mod memory;
    pub mod mutex;
    pub mod pci;
    pub mod pio;
    pub mod random;
    pub mod resource;
    pub mod scheduler;
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
pub static mut application: *mut Box<Application> = 0 as *mut Box<Application>;

#[no_mangle]
pub unsafe fn entry(){
    application = alloc(size_of::<Application>()) as *mut Box<Application>;
    if application as usize > 0 {
        ptr::write(application, box Application::new());
    }
}

#[no_mangle]
pub unsafe fn exit(){
    if application as usize > 0 {
        drop(ptr::read(application));

        unalloc(application as usize);
        application = 0 as *mut Box<Application>;
    }
}

#[no_mangle]
pub unsafe fn draw(display: &Display) -> bool{
    if application as usize > 0 {
        return (*application).draw(display);
    }else{
        return false;
    }
}

#[no_mangle]
pub unsafe fn on_key(key_event: KeyEvent){
    if application as usize > 0{
        (*application).on_key(key_event);
    }
}

#[no_mangle]
pub unsafe fn on_mouse(mouse_event: MouseEvent, allow_catch: bool) -> bool{
    if application as usize > 0 {
        return (*application).on_mouse(mouse_event, allow_catch);
    }else{
        return false;
    }
}

/* Externs { */
#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern fn __rust_allocate(size: usize, align: usize) -> *mut u8{
    return alloc(size) as *mut u8;
}

#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern fn __rust_deallocate(ptr: *mut u8, old_size: usize, align: usize){
    return unalloc(ptr as usize);
}

#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern fn __rust_reallocate(ptr: *mut u8, old_size: usize, size: usize, align: usize) -> *mut u8{
    return realloc(ptr as usize, size) as *mut u8;
}

#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern fn __rust_reallocate_inplace(ptr: *mut u8, old_size: usize, size: usize, align: usize) -> usize{
    return realloc_inplace(ptr as usize, size);
}

#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern fn __rust_usable_size(size: usize, align: usize) -> usize{
    return ((size + CLUSTER_SIZE - 1)/CLUSTER_SIZE) * CLUSTER_SIZE;
}

#[no_mangle]
pub unsafe extern fn memcmp(a: *mut u8, b: *const u8, len: usize) -> isize {
    for i in 0..len {
        let c_a = ptr::read(a.offset(i as isize));
        let c_b = ptr::read(b.offset(i as isize));
        if c_a != c_b{
            return c_a as isize - c_b as isize;
        }
    }
    return 0;
}

#[no_mangle]
pub unsafe extern fn memmove(dst: *mut u8, src: *const u8, len: usize){
    if src < dst {
        asm!("std
            rep movsb"
            :
            : "{edi}"(dst.offset(len as isize - 1)), "{esi}"(src.offset(len as isize - 1)), "{ecx}"(len)
            : "cc", "memory"
            : "intel");
    }else{
        asm!("cld
            rep movsb"
            :
            : "{edi}"(dst), "{esi}"(src), "{ecx}"(len)
            : "cc", "memory"
            : "intel");
    }
}

#[no_mangle]
pub unsafe extern fn memcpy(dst: *mut u8, src: *const u8, len: usize){
    asm!("cld
        rep movsb"
        :
        : "{edi}"(dst), "{esi}"(src), "{ecx}"(len)
        : "cc", "memory"
        : "intel");
}

#[no_mangle]
pub unsafe extern fn memset(dst: *mut u8, c: i32, len: usize) {
    asm!("cld
        rep stosb"
        :
        : "{eax}"(c), "{edi}"(dst), "{ecx}"(len)
        : "cc", "memory"
        : "intel");
}
/* } Externs */
