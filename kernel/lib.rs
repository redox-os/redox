//! # The Redox OS Kernel, version 2
//!
//! The Redox OS Kernel is a hybrid kernel that supports X86_64 systems and
//! provides Unix-like syscalls for primarily Rust applications

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
pub mod elf;

/// Schemes, filesystem handlers
pub mod scheme;

/// Synchronization primitives
pub mod sync;

/// Syscall handlers
pub mod syscall;

/// Tests
#[cfg(test)]
pub mod tests;

/// A unique number that identifies the current CPU - used for scheduling
#[thread_local]
static CPU_ID: AtomicUsize = ATOMIC_USIZE_INIT;

/// Get the current CPU's scheduling ID
#[inline(always)]
pub fn cpu_id() -> usize {
    CPU_ID.load(Ordering::Relaxed)
}

/// The count of all CPUs that can have work scheduled
static CPU_COUNT : AtomicUsize = ATOMIC_USIZE_INIT;

/// Get the number of CPUs currently active
#[inline(always)]
pub fn cpu_count() -> usize {
    CPU_COUNT.load(Ordering::Relaxed)
}

/// Initialize userspace by running the initfs:bin/init process
/// This function will also set the CWD to initfs:bin and open debug: as stdio
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

/// This is the kernel entry point for the primary CPU. The arch crate is responsible for calling this
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

/// This is the main kernel entry point for secondary CPUs
#[no_mangle]
pub extern fn kmain_ap(id: usize) {
    CPU_ID.store(id, Ordering::SeqCst);

    context::init();

    let pid = syscall::getpid();
    println!("AP {}: {:?}", id, pid);

    // Disable APs for now
    loop {
        unsafe { interrupt::enable_and_halt(); }
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
