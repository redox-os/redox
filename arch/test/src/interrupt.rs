//! Interrupt instructions

static mut INTERRUPTS_ENABLED: bool = false;

/// Clear interrupts
#[inline(always)]
pub unsafe fn disable() {
    println!("CLEAR INTERRUPTS");
    INTERRUPTS_ENABLED = false;
}

/// Set interrupts
#[inline(always)]
pub unsafe fn enable() {
    println!("SET INTERRUPTS");
    INTERRUPTS_ENABLED = true;
}

/// Halt instruction
#[inline(always)]
pub unsafe fn halt() {
    assert!(INTERRUPTS_ENABLED);
    ::std::thread::yield_now();
}

/// Pause instruction
#[inline(always)]
pub unsafe fn pause() {

}

/// Set interrupts and nop
#[inline(always)]
pub unsafe fn enable_and_nop() {
    enable();
}

/// Set interrupts and halt
#[inline(always)]
pub unsafe fn enable_and_halt() {
    enable();
    halt();
}
