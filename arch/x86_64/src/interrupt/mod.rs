//! Interrupt instructions

use core::mem;

use paging::{ActivePageTable, VirtualAddress};

pub mod exception;
pub mod irq;
pub mod syscall;

/// Clear interrupts
#[inline(always)]
pub unsafe fn disable() {
    asm!("cli" : : : : "intel", "volatile");
}

/// Set interrupts
#[inline(always)]
pub unsafe fn enable() {
    asm!("sti" : : : : "intel", "volatile");
}

/// Set interrupts and halt
#[inline(always)]
pub unsafe fn enable_and_halt() {
    asm!("sti
        hlt"
        : : : : "intel", "volatile");
}

/// Halt instruction
#[inline(always)]
pub unsafe fn halt() {
    asm!("hlt" : : : : "intel", "volatile");
}

/// Pause instruction
/// Safe because it is similar to a NOP, and has no memory effects
#[inline(always)]
pub fn pause() {
    unsafe { asm!("pause" : : : : "intel", "volatile"); }
}

/// Get a stack trace
//TODO: Check for stack being mapped before dereferencing
#[inline(never)]
pub unsafe fn stack_trace() {
    let mut rbp: usize;
    asm!("" : "={rbp}"(rbp) : : : "intel", "volatile");

    println!("TRACE: {:>016X}", rbp);
    //Maximum 64 frames
    let active_table = ActivePageTable::new();
    for _frame in 0..64 {
        if active_table.translate(VirtualAddress::new(rbp)).is_some() && active_table.translate(VirtualAddress::new(rbp + mem::size_of::<usize>())).is_some() {
            let rip = *(rbp as *const usize).offset(1);
            println!("  {:>016X}: {:>016X}", rbp, rip);
            rbp = *(rbp as *const usize);
        } else {
            println!("  {:>016X}: GUARD PAGE", rbp);
            break;
        }
    }
}
