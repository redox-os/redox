#![feature(asm)]
#![feature(lang_items)]
#![no_std]

#[macro_use]
extern crate syscall;

use syscall::{exit, write};

#[no_mangle]
pub unsafe extern fn _start() {
    let string = b"Hello, World!\n";
    write(1, string);
    exit(0);
}

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[cfg(not(test))]
/// Required to handle panics
#[lang = "panic_fmt"]
extern "C" fn panic_fmt(_fmt: ::core::fmt::Arguments, _file: &str, _line: u32) -> ! {
    write(2, b"panic\n");
    exit(127);
    loop {}
}

/// Memcpy
///
/// Copy N bytes of memory from one location to another.
#[no_mangle]
pub unsafe extern fn memcpy(dest: *mut u8, src: *const u8,
                            n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.offset(i as isize) = *src.offset(i as isize);
        i += 1;
    }

    dest
}

/// Memmove
///
/// Copy N bytes of memory from src to dest. The memory areas may overlap.
#[no_mangle]
pub unsafe extern fn memmove(dest: *mut u8, src: *const u8,
                             n: usize) -> *mut u8 {
    if src < dest as *const u8 {
        let mut i = n;
        while i != 0 {
            i -= 1;
            *dest.offset(i as isize) = *src.offset(i as isize);
        }
    } else {
        let mut i = 0;
        while i < n {
            *dest.offset(i as isize) = *src.offset(i as isize);
            i += 1;
        }
    }

    dest
}

/// Memset
///
/// Fill a block of memory with a specified value.
#[no_mangle]
pub unsafe extern fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.offset(i as isize) = c as u8;
        i += 1;
    }

    s
}

/// Memcmp
///
/// Compare two blocks of memory.
#[no_mangle]
pub unsafe extern fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0;

    while i < n {
        let a = *s1.offset(i as isize);
        let b = *s2.offset(i as isize);
        if a != b {
            return a as i32 - b as i32
        }
        i += 1;
    }

    0
}
