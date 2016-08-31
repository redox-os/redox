//! Interrupt instructions

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
    asm!("xchg bx, bx" : "={rbp}"(rbp) : : : "intel", "volatile");

    println!("TRACE: {:>016X}", rbp);
    //Maximum 64 frames
    for _frame in 0..64 {
        let rip = *(rbp as *const usize).offset(1);
        println!("  {:>016X}: {:>016X}", rbp, rip);
        if rip == 0 {
            break;
        }
        rbp = *(rbp as *const usize);
    }
}
