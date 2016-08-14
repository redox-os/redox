/// This function is where the kernel sets up IRQ handlers
/// It is increcibly unsafe, and should be minimal in nature
/// It must create the IDT with the correct entries, those entries are
/// defined in other files inside of the `arch` module

use super::idt::{IDTR, IDT};

#[naked]
#[no_mangle]
pub unsafe extern "C" fn kmain() {
    asm!("xchg bx, bx" : : : : "intel", "volatile");

    for entry in IDT.iter_mut() {
        entry.attribute = 1 << 7 | 0xE;
        entry.selector = 8;
        entry.set_offset(blank as usize);
        entry.zero = 0;
        entry.zero2 = 0;
    }
    IDTR.set_slice(&IDT);
    asm!("lidt [rax]" : : "{rax}"(&IDTR as *const _ as usize) : : "intel", "volatile");

    asm!("xchg bx, bx" : : : : "intel", "volatile");

    asm!("int 0xFF" : : : : "intel", "volatile");

    loop{
        asm!("hlt" : : : : "intel", "volatile");
    }
}

#[naked]
pub unsafe extern "C" fn blank() {
    asm!("xchg bx, bx" : : : : "intel", "volatile");
    asm!("iretq" : : : : "intel", "volatile");
}
