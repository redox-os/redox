//! # The Redox OS Kernel, version 2
//!
//! The Redox OS Kernel is a hybrid kernel that supports X86 systems and
//! provides Unix-like syscalls for primarily Rust applications
//!
//! ## Syscalls
//! Syscalls in Redox are often handled by userspace `schemes`.
//! The essential syscalls in Redox are as follows:
//!
//! ### Open
//! `open(path: &str, flags: usize) -> Result<file_descriptor: usize>`
//!
//! Open a file, providing a path as a `&str` and flags, defined elsewhere.
//!
//! Returns a number, known as a file descriptor, that is passed to other syscalls
//!
//! ### Close
//! `close(file_descriptor: usize) -> Result<()>`
//!
//! Close a file descriptor, providing the file descriptor from `open`
//!
//! Returns an error, `EBADF`, if the file descriptor was not found.
//!
//! This potential error is often ignored by userspace
//!
//! ### Duplicate
//! `dup(file_descriptor: usize) -> Result<file_descriptor: usize>`
//!
//! Duplicate a file descriptor, providing the file descriptor from `open`
//!
//! Returns a new file descriptor, or an error
//!
//! ### Read
//! `read(file_descriptor: usize, buffer: &mut [u8]) -> Result<count: usize>`
//!
//! Read from a file descriptor, providing the file descriptor from `open` and a mutable buffer
//!
//! Returns the number of bytes actually read, or an error
//!
//! ### Write
//! `write(file_descriptor: usize, buffer: &[u8]) -> Result<count: usize>`
//!
//! Write to a file descriptor, providing the file descriptor from `open` and a const buffer
//!
//! Returns the number of bytes actually written, or an error
//!
//! ### Stat
//! `fstat(file_descriptor: usize, stat: &mut Stat) -> Result<()>`
//!
//! Get information from a file descriptor, providing the file descriptor from `open`
//! and a mutable Stat struct, defined elsewhere.
//!
//! Returns an error if the operation failed
//!
//! ### Path
//! `fpath(file_descriptor: usize, buffer: &mut [u8]) -> Result<count: usize>`
//!
//! Read the path of a file descriptor, providing the file descriptor from `open` and
//! a mutable buffer.
//!
//! Returns the number of bytes actually read, or an error
//!
//! The buffer should be 4096 bytes, to ensure that the entire path will fit.
//! An error will be returned, `ENOBUFS`, if the buffer is not long enough for the name.
//! In this case, it is recommended to add one page, 4096 bytes, to the buffer and retry.

#![feature(const_fn)]
#![feature(lang_items)]
#![feature(question_mark)]
#![no_std]

#[macro_use]
extern crate bitflags;

use arch::interrupt::{enable_interrupts, halt};

/// Architecture specific items (test)
#[cfg(test)]
#[macro_use]
extern crate arch_test as arch;

/// Architecture specific items (x86_64)
#[cfg(all(not(test), target_arch = "x86_64"))]
#[macro_use]
extern crate arch_x86_64 as arch;

/// Context management
pub mod context;

/// Intrinsics for panic handling
pub mod panic;

/// Syscall handlers
pub mod syscall;

/// Tests
#[cfg(test)]
pub mod tests;

#[no_mangle]
pub extern fn kmain() {
    println!("TEST");

    println!("{:?}", syscall::open(b"stdin", 0));
    println!("{:?}", syscall::open(b"stdout", 0));
    println!("{:?}", syscall::open(b"stderr", 0));

    loop {
        unsafe {
            enable_interrupts();
            halt();
        }
        println!("{:?}", syscall::write(1, b"HALT\n"));
    }
}
