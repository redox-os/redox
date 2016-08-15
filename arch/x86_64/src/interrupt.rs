//! Interrupt instructions

/// Clear interrupts
#[inline(always)]
pub unsafe fn disable_interrupts() {
    asm!("cli" : : : : "intel", "volatile");
}

/// Set interrupts
#[inline(always)]
pub unsafe fn enable_interrupts() {
    asm!("sti" : : : : "intel", "volatile");
}

/// Halt instruction
#[inline(always)]
pub unsafe fn halt() {
    asm!("hlt" : : : : "intel", "volatile");
}
