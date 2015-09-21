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

use application::main;

use common::memory::*;

use programs::common::*;

#[path="APPLICATION_PATH"]
mod application;

#[path="src/audio"]
mod audio {
    pub mod wav;
}

#[path="src/common"]
mod common {
    pub mod debug;
    pub mod event;
    pub mod queue;
    pub mod memory;
    pub mod random;
    pub mod resource;
    pub mod scheduler;
    pub mod string;
    pub mod time;
    pub mod vec;
}

#[path="src/graphics"]
mod graphics {
    pub mod bmp;
    pub mod color;
    pub mod consolewindow;
    pub mod display;
    pub mod point;
    pub mod size;
    pub mod window;
}

#[path="src/programs"]
mod programs {
    pub mod common;
}

#[path="src/syscall"]
mod syscall {
    pub mod call;
    pub mod common;
}

#[no_mangle]
pub unsafe fn _start(){
    main();
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
            : "intel", "volatile");
    }else{
        asm!("cld
            rep movsb"
            :
            : "{edi}"(dst), "{esi}"(src), "{ecx}"(len)
            : "cc", "memory"
            : "intel", "volatile");
    }
}

#[no_mangle]
pub unsafe extern fn memcpy(dst: *mut u8, src: *const u8, len: usize){
    asm!("cld
        rep movsb"
        :
        : "{edi}"(dst), "{esi}"(src), "{ecx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}

#[no_mangle]
pub unsafe extern fn memset(dst: *mut u8, c: i32, len: usize) {
    asm!("cld
        rep stosb"
        :
        : "{eax}"(c), "{edi}"(dst), "{ecx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}
/* } Externs */
