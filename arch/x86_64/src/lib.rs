//! Architecture support for x86_64

#![feature(asm)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]
#![no_std]

#[macro_use]
extern crate bitflags;

/// Print to console
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!($crate::serial::SerialConsole::new(), $($arg)*);
    });
}

/// Print with new line to console
#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// Global descriptor table
pub mod gdt;

/// Interrupt descriptor table
pub mod idt;

/// IO Handling
pub mod io;

/// IRQ Handling
pub mod irq;

/// Interrupt instructions
pub mod interrupt;

/// Initialization and main function
pub mod main;

/// Memcpy, memmove, etc.
pub mod mem;

/// Serial driver and print! support
pub mod serial;

/// Task state segment
pub mod tss;

pub mod physical;
