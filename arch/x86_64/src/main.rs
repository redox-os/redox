/// This function is where the kernel sets up IRQ handlers
/// It is increcibly unsafe, and should be minimal in nature
/// It must create the IDT with the correct entries, those entries are
/// defined in other files inside of the `arch` module

use super::idt::{IDTR, IDT, IDT_PRESENT, IDT_RING_0, IDT_INTERRUPT};
use super::mem::memset;

extern {
    /// The starting byte of the text (code) data segment.
    static mut __text_start: u8;
    /// The ending byte of the text (code) data segment.
    static mut __text_end: u8;
    /// The starting byte of the _.rodata_ (read-only data) segment.
    static mut __rodata_start: u8;
    /// The ending byte of the _.rodata_ (read-only data) segment.
    static mut __rodata_end: u8;
    /// The starting byte of the _.data_ segment.
    static mut __data_start: u8;
    /// The ending byte of the _.data_ segment.
    static mut __data_end: u8;
    /// The starting byte of the _.bss_ (uninitialized data) segment.
    static mut __bss_start: u8;
    /// The ending byte of the _.bss_ (uninitialized data) segment.
    static mut __bss_end: u8;
}

/// Test of zero values in BSS.
static BSS_TEST_ZERO: usize = 0;
/// Test of non-zero values in BSS.
static BSS_TEST_NONZERO: usize = 0xFFFFFFFFFFFFFFFF;

extern {
    fn kmain() -> !;
}

#[no_mangle]
pub unsafe extern fn kstart() -> ! {
    asm!("xchg bx, bx" : : : : "intel", "volatile");

    // Zero BSS, this initializes statics that are set to 0
    {
        let start_ptr = &mut __bss_start as *mut u8;
        let end_ptr = & __bss_end as *const u8 as usize;

        if start_ptr as usize <= end_ptr {
            let size = end_ptr - start_ptr as usize;
            memset(start_ptr, 0, size);
        }

        debug_assert_eq!(BSS_TEST_ZERO, 0);
        debug_assert_eq!(BSS_TEST_NONZERO, 0xFFFFFFFFFFFFFFFF);
    }

    asm!("xchg bx, bx" : : : : "intel", "volatile");

    //Set up IDT
    for entry in IDT.iter_mut() {
        entry.set_flags(IDT_PRESENT | IDT_RING_0 | IDT_INTERRUPT);
        entry.set_offset(8, blank as usize);
    }
    IDTR.set_slice(&IDT);
    IDTR.load();

    asm!("xchg bx, bx" : : : : "intel", "volatile");

    asm!("int 0xFF" : : : : "intel", "volatile");

    asm!("xchg bx, bx" : : : : "intel", "volatile");

    kmain();
}

interrupt!(blank, {
    println!("INTERRUPT");
});
