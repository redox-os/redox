use arch::interrupt::{set_interrupts, halt};

#[test]
fn halt_with_interrupts() {
    unsafe {
        set_interrupts();
        halt();
    }
}
