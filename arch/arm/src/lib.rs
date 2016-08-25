//! Architecture support for ARM

#![feature(asm)]
#![feature(lang_items)]
#![feature(naked_functions)]
#![no_std]

extern crate hole_list_allocator as allocator;
#[macro_use]
extern crate bitflags;
extern crate spin;

/// Print to console
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({});
}

/// Print with new line to console
#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// Context switching
pub mod context;

/// Memset, memcpy, etc.
pub mod externs;

/// Interrupt handling
pub mod interrupt;

/// Panic support
pub mod panic;

/// Initialization function
pub mod start;
