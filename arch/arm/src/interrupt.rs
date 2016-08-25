//! Interrupt instructions

/// Clear interrupts
#[inline(always)]
pub unsafe fn disable() {
}

/// Set interrupts
#[inline(always)]
pub unsafe fn enable() {
}

/// Set interrupts and halt
#[inline(always)]
pub unsafe fn enable_and_halt() {
    halt();
}

/// Halt instruction
#[inline(always)]
pub unsafe fn halt() {
    //asm!("wfi" : : : : "volatile");
    asm!("nop" : : : : "volatile");
}

/// Get a stack trace
//TODO: Check for stack being mapped before dereferencing
#[inline(never)]
pub unsafe fn stack_trace() {
}
