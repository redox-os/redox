use core::fmt;
use core::ptr;

use common::debug::*;

use syscall::call::*;

#[lang="stack_exhausted"]
extern "C" fn stack_exhausted() {

}

#[lang="eh_personality"]
extern "C" fn eh_personality() {

}

#[lang = "panic_fmt"]
pub fn panic_fmt(fmt: fmt::Arguments, file: &'static str, line: u32) -> ! {
    d(file);
    d(": ");
    dh(line as usize);
    dl();
    unsafe {
        sys_exit(-1);
        loop {
            asm!("sti");
            asm!("hlt");
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *mut u8, b: *const u8, len: usize) -> isize {
    for i in 0..len {
        let c_a = ptr::read(a.offset(i as isize));
        let c_b = ptr::read(b.offset(i as isize));
        if c_a != c_b {
            return c_a as isize - c_b as isize;
        }
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, len: usize) {
    if src < dst {
        asm!("std
            rep movsb"
            :
            : "{edi}"(dst.offset(len as isize - 1)), "{esi}"(src.offset(len as isize - 1)), "{ecx}"(len)
            : "cc", "memory"
            : "intel", "volatile");
    } else {
        asm!("cld
            rep movsb"
            :
            : "{edi}"(dst), "{esi}"(src), "{ecx}"(len)
            : "cc", "memory"
            : "intel", "volatile");
    }
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, len: usize) {
    asm!("cld
        rep movsb"
        :
        : "{edi}"(dst), "{esi}"(src), "{ecx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}

#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut u8, c: i32, len: usize) {
    asm!("cld
        rep stosb"
        :
        : "{eax}"(c), "{edi}"(dst), "{ecx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}

/*
pub fn unsupported() {
    unsafe { asm!("int 3" : : : : "intel", "volatile") }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn fmod(x: f64, y: f64) -> f64 {
    unsupported();
    return 0.0;
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn fmodf(x: f32, y: f32) -> f32 {
    unsupported();
    return 0.0;
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn __powisf2(a: f32, x: i32) -> f32 {
    unsupported();
    return 0.0;
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn __powidf2(a: f64, x: i32) -> f64 {
    unsupported();
    return 0.0;
}

#[no_mangle]
pub extern fn __mulodi4(a: i32, b: i32, overflow: *mut i32) -> i32 {
    let result = (a as i64) * (b as i64);
    if result > 2 << 32 {
        unsafe {
            ptr::write(overflow, 1);
        }
    }
    return result as i32;
}

#[no_mangle]
pub extern fn __moddi3(a: i32, b: i32) -> i32 {
    return a%b;
}

#[no_mangle]
pub extern fn __divdi3(a: i32, b: i32) -> i32 {
    return a/b;
}

#[no_mangle]
pub extern fn __umoddi3(a: u32, b: u32) -> u32 {
    return a%b;
}

#[no_mangle]
pub extern fn __udivdi3(a: u32, b: u32) -> u32 {
    return a/b;
}
*/
