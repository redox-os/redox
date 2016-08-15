/// This function is where the kernel sets up IRQ handlers
/// It is increcibly unsafe, and should be minimal in nature
/// It must create the IDT with the correct entries, those entries are
/// defined in other files inside of the `arch` module

use externs::memset;
use idt;
use memory::{self, Frame};
use paging::{self, entry, PhysicalAddress};

/// Test of zero values in BSS.
static BSS_TEST_ZERO: usize = 0;
/// Test of non-zero values in BSS.
static BSS_TEST_NONZERO: usize = 0xFFFFFFFFFFFFFFFF;

extern {
    /// Kernel main function
    fn kmain() -> !;
}

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

    // Set up IDT
    idt::init(blank);

    // Initialize memory management
    let mut allocator = memory::init(0, &__bss_end as *const u8 as usize);

    // Initialize paging
    let mut pager = paging::init();

    // Remap a section with `flags`
    let mut remap_section = |start_ref: &u8, end_ref: &u8, flags: entry::EntryFlags| {
        let start = start_ref as *const _ as usize;
        let end = end_ref as *const _ as usize;

        for i in 0..(start - end + paging::PAGE_SIZE - 1)/paging::PAGE_SIZE {
            let frame = Frame::containing_address(PhysicalAddress::new(start + i * paging::PAGE_SIZE));
            pager.identity_map(frame, flags, &mut allocator);
        }
    };

    // Remap text read-only
    {
        asm!("xchg bx, bx" : : : : "intel", "volatile");
        //TODO remap_section(& __text_start, & __text_end, entry::PRESENT);
    }

    kmain();
}

interrupt!(blank, {
    println!("INTERRUPT");
});
