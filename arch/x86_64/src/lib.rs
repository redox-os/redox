//! Architecture support for x86_64

#![feature(asm)]
#![feature(concat_idents)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(naked_functions)]
#![feature(unique)]
#![no_std]

extern crate hole_list_allocator as allocator;
#[macro_use]
extern crate bitflags;
extern crate spin;
extern crate x86;

/// Print to console
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!($crate::serial::SerialConsole, $($arg)*);
    });
}

/// Print with new line to console
#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// Create an interrupt function that can safely run rust code
#[macro_export]
macro_rules! interrupt {
    ($name:ident, $func:block) => {
        #[naked]
        pub unsafe extern fn $name () {
            #[inline(never)]
            unsafe fn inner() {
                $func
            }

            // Push scratch registers
            asm!("push rax
                push rcx
                push rdx
                push rdi
                push rsi
                push r8
                push r9
                push r10
                push r11"
                : : : : "intel", "volatile");

            // Call inner rust function
            inner();

            // Pop scratch registers and return
            asm!("pop r11
                pop r10
                pop r9
                pop r8
                pop rsi
                pop rdi
                pop rdx
                pop rcx
                pop rax
                iretq"
                : : : : "intel", "volatile");
        }
    };
}

#[macro_export]
macro_rules! interrupt_error {
    ($name:ident, $func:block) => {
        #[naked]
        pub unsafe extern fn $name () {
            #[inline(never)]
            unsafe fn inner() {
                $func
            }

            // Push scratch registers
            asm!("push rax
                push rcx
                push rdx
                push rdi
                push rsi
                push r8
                push r9
                push r10
                push r11"
                : : : : "intel", "volatile");


            // Call inner rust function
            inner();

            // Pop scratch registers, error code, and return
            asm!("pop r11
                pop r10
                pop r9
                pop r8
                pop rsi
                pop rdi
                pop rdx
                pop rcx
                pop rax
                add rsp, 8
                iretq"
                : : : : "intel", "volatile");
        }
    };
}

/// ACPI table parsing
pub mod acpi;

/// Memcpy, memmove, etc.
pub mod externs;

/// Global descriptor table
pub mod gdt;

/// Interrupt descriptor table
pub mod idt;

/// IO Handling
pub mod io;

/// Interrupt instructions
pub mod interrupt;

/// Memory management
pub mod memory;

/// Paging
pub mod paging;

/// Panic
pub mod panic;

/// Serial driver and print! support
pub mod serial;

/// Initialization and start function
pub mod start;

/// Task state segment
pub mod tss;
