/// This function is where the kernel sets up IRQ handlers
/// It is increcibly unsafe, and should be minimal in nature
/// It must create the IDT with the correct entries, those entries are
/// defined in other files inside of the `arch` module
#[naked]
#[no_mangle]
pub unsafe extern fn kmain() {
    asm!("xchg bx, bx" : : : : "intel", "volatile");

    loop{}
}
