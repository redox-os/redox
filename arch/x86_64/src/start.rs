/// This function is where the kernel sets up IRQ handlers
/// It is increcibly unsafe, and should be minimal in nature
/// It must create the IDT with the correct entries, those entries are
/// defined in other files inside of the `arch` module

use core::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

use acpi;
use allocator::{HEAP_START, HEAP_SIZE};
use device;
use externs::memset;
use gdt;
use idt;
use interrupt;
use memory::{self, Frame};
use paging::{self, entry, Page, PhysicalAddress, VirtualAddress};

/// Test of zero values in BSS.
static BSS_TEST_ZERO: usize = 0;
/// Test of non-zero values in data.
static DATA_TEST_NONZERO: usize = 0xFFFFFFFFFFFFFFFF;
/// Test of zero values in thread BSS
#[thread_local]
static mut TBSS_TEST_ZERO: usize = 0;
/// Test of non-zero values in thread data.
#[thread_local]
static mut TDATA_TEST_NONZERO: usize = 0xFFFFFFFFFFFFFFFF;

static AP_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;
pub static AP_READY: AtomicBool = ATOMIC_BOOL_INIT;
static BSP_READY: AtomicBool = ATOMIC_BOOL_INIT;
static HEAP_FRAME: AtomicUsize = ATOMIC_USIZE_INIT;

extern {
    /// Kernel main function
    fn kmain() -> !;
    /// Kernel main for APs
    fn kmain_ap(id: usize) -> !;
}

/// The entry to Rust, all things must be initialized
#[no_mangle]
pub unsafe extern fn kstart() -> ! {
    {
        extern {
            /// The starting byte of the _.bss_ (uninitialized data) segment.
            static mut __bss_start: u8;
            /// The ending byte of the _.bss_ (uninitialized data) segment.
            static mut __bss_end: u8;
            /// The end of the tbss.
            static mut __tbss_end: u8;
            /// The end of the kernel
            static mut __end: u8;
        }

        // Zero BSS, this initializes statics that are set to 0
        {
            let start_ptr = &mut __bss_start as *mut u8;
            let end_ptr = & __bss_end as *const u8 as usize;

            if start_ptr as usize <= end_ptr {
                let size = end_ptr - start_ptr as usize;
                memset(start_ptr, 0, size);
            }

            assert_eq!(BSS_TEST_ZERO, 0);
            assert_eq!(DATA_TEST_NONZERO, 0xFFFFFFFFFFFFFFFF);
        }

        // Initialize memory management
        memory::init(0, &__end as *const u8 as usize);

        // TODO: allocate a stack
        let stack_start = 0x00080000;
        let stack_end = 0x0009F000;

        // Initialize paging
        let mut active_table = paging::init(stack_start, stack_end);

        // Set up GDT
        gdt::init((&__tbss_end as *const u8 as *const usize).offset(-1) as usize, stack_end);

        // Set up IDT
        idt::init();

        // Test tdata and tbss
        {
            assert_eq!(TBSS_TEST_ZERO, 0);
            TBSS_TEST_ZERO += 1;
            assert_eq!(TBSS_TEST_ZERO, 1);
            assert_eq!(TDATA_TEST_NONZERO, 0xFFFFFFFFFFFFFFFF);
            TDATA_TEST_NONZERO -= 1;
            assert_eq!(TDATA_TEST_NONZERO, 0xFFFFFFFFFFFFFFFE);
        }

        // Reset AP variables
        AP_COUNT.store(0, Ordering::SeqCst);
        AP_READY.store(false, Ordering::SeqCst);
        BSP_READY.store(false, Ordering::SeqCst);
        HEAP_FRAME.store(0, Ordering::SeqCst);

        // Map heap
        {
            let heap_start_page = Page::containing_address(VirtualAddress::new(HEAP_START));
            let heap_end_page = Page::containing_address(VirtualAddress::new(HEAP_START + HEAP_SIZE-1));

            {
                let index = heap_start_page.p4_index();
                assert_eq!(index, heap_end_page.p4_index());

                let frame = memory::allocate_frame().expect("no frames available");
                HEAP_FRAME.store(frame.start_address().get(), Ordering::SeqCst);

                let p4 = active_table.p4_mut();
                {
                    let entry = &mut p4[index];
                    assert!(entry.is_unused());
                    entry.set(frame, entry::PRESENT | entry::WRITABLE);
                }
                p4.next_table_mut(index).unwrap().zero();
            }

            for page in Page::range_inclusive(heap_start_page, heap_end_page) {
                active_table.map(page, entry::WRITABLE | entry::NO_EXECUTE);
            }
        }

        // Initialize devices
        device::init(&mut active_table);

        // Read ACPI tables, starts APs
        acpi::init(&mut active_table);

        BSP_READY.store(true, Ordering::SeqCst);
    }

    kmain();
}

/// Entry to rust for an AP
pub unsafe extern fn kstart_ap(stack_start: usize, stack_end: usize) -> ! {
    {
        extern {
            /// The end of the tbss.
            static mut __tbss_end: u8;
        }

        assert_eq!(BSS_TEST_ZERO, 0);
        assert_eq!(DATA_TEST_NONZERO, 0xFFFFFFFFFFFFFFFF);

        // Initialize paging
        let mut active_table = paging::init(stack_start, stack_end);

        // Set up GDT for AP
        gdt::init_ap((&__tbss_end as *const u8 as *const usize).offset(-1) as usize, stack_end);

        // Set up IDT for AP
        idt::init();

        // Test tdata and tbss
        {
            assert_eq!(TBSS_TEST_ZERO, 0);
            TBSS_TEST_ZERO += 1;
            assert_eq!(TBSS_TEST_ZERO, 1);
            assert_eq!(TDATA_TEST_NONZERO, 0xFFFFFFFFFFFFFFFF);
            TDATA_TEST_NONZERO -= 1;
            assert_eq!(TDATA_TEST_NONZERO, 0xFFFFFFFFFFFFFFFE);
        }

        // Map heap
        {
            let heap_start_page = Page::containing_address(VirtualAddress::new(HEAP_START));
            let heap_end_page = Page::containing_address(VirtualAddress::new(HEAP_START + HEAP_SIZE-1));

            {
                assert_eq!(heap_start_page.p4_index(), heap_end_page.p4_index());

                while HEAP_FRAME.load(Ordering::SeqCst) == 0 {
                    interrupt::pause();
                }
                let frame = Frame::containing_address(PhysicalAddress::new(HEAP_FRAME.load(Ordering::SeqCst)));

                let p4 = active_table.p4_mut();
                let entry = &mut p4[heap_start_page.p4_index()];
                assert!(entry.is_unused());
                entry.set(frame, entry::PRESENT | entry::WRITABLE);
            }
        }

        // Init devices for AP
        device::init_ap(&mut active_table);

        AP_READY.store(true, Ordering::SeqCst);
    }

    let ap_number = AP_COUNT.fetch_add(1, Ordering::SeqCst);

    while ! BSP_READY.load(Ordering::SeqCst) {
        interrupt::pause();
    }

    kmain_ap(ap_number);
}

pub unsafe fn usermode(ip: usize, sp: usize) -> ! {
    // Go to usermode
    asm!("mov rax, 0x2B # Set segment pointers
        mov ds, ax
        mov es, ax
        mov fs, ax
        mov gs, ax

        push rax # Push stack segment
        push rbx # Push stack pointer
        mov rax, 3 << 12 | 1 << 9 # Set IOPL and interrupt enable flag
        push rax # Push rflags
        mov rax, 0x23
        push rax # Push code segment
        push rcx # Push rip
        xchg bx, bx
        iretq"
        :
        : "{rbx}"(sp), "{rcx}"(ip)
        : "rax", "sp"
        : "intel", "volatile");
    unreachable!();
}
