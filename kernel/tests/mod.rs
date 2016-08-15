use arch::interrupt::{enable_interrupts, halt};

#[test]
fn halt_with_interrupts() {
    unsafe {
        enable_interrupts();
        halt();
    }
}
