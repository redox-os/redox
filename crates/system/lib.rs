#![crate_name="system"]
#![crate_type="lib"]
#![feature(asm)]
#![feature(lang_items)]
#![no_std]

use core::{ptr, slice, str};

pub mod error;
#[cfg(target_os="redox")]
pub mod externs;
pub mod scheme;
pub mod syscall;

/// Helper function for handling C strings, please do not copy it or make it pub or change it
pub fn c_string_to_slice<'a>(ptr: *const u8) -> &'a [u8] {
    if ptr > 0 as *const u8 {
        let mut len = 0;
        while unsafe { ptr::read(ptr.offset(len as isize)) } > 0 {
            len += 1;
        }

        unsafe { slice::from_raw_parts(ptr, len) }
    } else {
        &[]
    }
}

pub fn c_string_to_str<'a>(ptr: *const u8) -> &'a str {
    unsafe { str::from_utf8_unchecked(c_string_to_slice(ptr)) }
}

/// Helper function for handling C strings, please do not copy it or make it pub or change it
pub fn c_array_to_slice<'a>(ptr: *const *const u8) -> &'a [*const u8] {
    if ptr > 0 as *const *const u8 {
        let mut len = 0;
        while unsafe { ptr::read(ptr.offset(len as isize)) } > 0 as *const u8 {
            len += 1;
        }

        unsafe { slice::from_raw_parts(ptr, len) }
    } else {
        &[]
    }
}
