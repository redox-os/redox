/// This function is where the kernel sets up IRQ handlers
/// It is increcibly unsafe, and should be minimal in nature
/// It must create the IDT with the correct entries, those entries are
/// defined in other files inside of the `arch` module

use allocator::{HEAP_START, HEAP_SIZE};
use externs::memset;
use gdt;
use idt;
use memory;
use paging::{self, Page, VirtualAddress};

/// Test of zero values in BSS.
static BSS_TEST_ZERO: usize = 0;
/// Test of non-zero values in BSS.
static BSS_TEST_NONZERO: usize = 0xFFFFFFFFFFFFFFFF;

extern {
    /// Kernel main function
    fn kmain() -> !;
}

#[no_mangle]
pub unsafe extern fn kstart() -> ! {
    extern {
        /// The starting byte of the _.bss_ (uninitialized data) segment.
        static mut __bss_start: u8;
        /// The ending byte of the _.bss_ (uninitialized data) segment.
        static mut __bss_end: u8;
    }

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

    // Set up GDT
    gdt::init();

    // Set up IDT
    idt::init(blank);

    // Initialize memory management
    let mut allocator = memory::init(0, &__bss_end as *const u8 as usize);

    // Initialize paging
    let mut active_table = paging::init(&mut allocator);

    // Initialize heap
    let heap_start_page = Page::containing_address(VirtualAddress::new(HEAP_START));
    let heap_end_page = Page::containing_address(VirtualAddress::new(HEAP_START + HEAP_SIZE-1));

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        active_table.map(page, paging::entry::WRITABLE, &mut allocator);
    }

    asm!("xchg bx, bx" : : : : "intel", "volatile");
    kmain();
}

interrupt!(blank, {
    asm!("xchg bx, bx" : : : : "intel", "volatile");
    println!("INTERRUPT");
});
