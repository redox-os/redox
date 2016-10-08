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

#![feature(alloc)]
#![feature(arc_counts)]
#![feature(asm)]
#![feature(collections)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(drop_types_in_const)]
#![feature(heap_api)]
#![feature(integer_atomics)]
#![feature(never_type)]
#![feature(thread_local)]
#![no_std]

use arch::interrupt;

/// Architecture specific items (test)
#[cfg(test)]
#[macro_use]
extern crate arch_test as arch;

/// Architecture specific items (ARM)
#[cfg(all(not(test), target_arch = "arm"))]
#[macro_use]
extern crate arch_arm as arch;

/// Architecture specific items (x86_64)
#[cfg(all(not(test), target_arch = "x86_64"))]
#[macro_use]
extern crate arch_x86_64 as arch;

extern crate alloc;
#[macro_use]
extern crate collections;

#[macro_use]
extern crate bitflags;
extern crate goblin;
extern crate spin;

use core::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

/// Context management
pub mod context;

/// ELF file parsing
#[cfg(all(not(test), target_arch = "x86_64"))]
pub mod elf;

/// Schemes, filesystem handlers
pub mod scheme;

/// Synchronization primitives
pub mod sync;

/// Syscall handlers
pub mod syscall;

pub mod capability;

/// Tests
#[cfg(test)]
pub mod tests;

#[thread_local]
static CPU_ID: AtomicUsize = ATOMIC_USIZE_INIT;

#[inline(always)]
pub fn cpu_id() -> usize {
    CPU_ID.load(Ordering::Relaxed)
}

static CPU_COUNT : AtomicUsize = ATOMIC_USIZE_INIT;

#[inline(always)]
pub fn cpu_count() -> usize {
    CPU_COUNT.load(Ordering::Relaxed)
}

pub extern fn userspace_init() {
    assert_eq!(syscall::chdir(b"initfs:bin"), Ok(0));

    assert_eq!(syscall::open(b"debug:", 0), Ok(0));
    assert_eq!(syscall::open(b"debug:", 0), Ok(1));
    assert_eq!(syscall::open(b"debug:", 0), Ok(2));

    syscall::exec(b"initfs:bin/init", &[]).expect("failed to execute initfs:init");

    panic!("initfs:init returned")
}

/// Allow exception handlers to send signal to arch-independant kernel
#[no_mangle]
pub extern fn ksignal(signal: usize) {
    println!("SIGNAL {}, CPU {}, PID {}", signal, cpu_id(), context::context_id());
    {
        let contexts = context::contexts();
        if let Some(context_lock) = contexts.current() {
            let context = context_lock.read();
            println!("NAME {}", unsafe { ::core::str::from_utf8_unchecked(&context.name.lock()) });
        }
    }
}

#[no_mangle]
pub extern fn kmain(cpus: usize) {
    CPU_ID.store(0, Ordering::SeqCst);
    CPU_COUNT.store(cpus, Ordering::SeqCst);

    context::init();

    let pid = syscall::getpid();
    println!("BSP: {:?} {}", pid, cpus);

    match context::contexts_mut().spawn(userspace_init) {
        Ok(context_lock) => {
            let mut context = context_lock.write();
            context.status = context::Status::Runnable;
        },
        Err(err) => {
            panic!("failed to spawn userspace_init: {:?}", err);
        }
    }

    loop {
        unsafe {
            interrupt::disable();
            if context::switch() {
                interrupt::enable_and_nop();
            } else {
                // Enable interrupts, then halt CPU (to save power) until the next interrupt is actually fired.
                interrupt::enable_and_halt();
            }
        }
    }
}

#[no_mangle]
pub extern fn kmain_ap(id: usize) {
    CPU_ID.store(id, Ordering::SeqCst);

    context::init();

    let pid = syscall::getpid();
    println!("AP {}: {:?}", id, pid);

    loop {
        unsafe {
            interrupt::disable();
            if context::switch() {
                interrupt::enable_and_nop();
            } else {
                // Enable interrupts, then halt CPU (to save power) until the next interrupt is actually fired.
                interrupt::enable_and_halt();
            }
        }
    }
}
