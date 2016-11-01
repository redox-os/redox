/// This function is where the kernel sets up IRQ handlers
/// It is increcibly unsafe, and should be minimal in nature
/// It must create the IDT with the correct entries, those entries are
/// defined in other files inside of the `arch` module

use core::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

use acpi;
use allocator;
use device;
use externs::memset;
use gdt;
use idt;
use interrupt;
use memory;
use paging::{self, entry, Page, VirtualAddress};

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

pub static CPU_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;
pub static AP_READY: AtomicBool = ATOMIC_BOOL_INIT;
static BSP_READY: AtomicBool = ATOMIC_BOOL_INIT;

extern {
    /// Kernel main function
    fn kmain(cpus: usize) -> !;
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
        memory::init(0, &__end as *const u8 as usize - ::KERNEL_OFFSET);

        // TODO: allocate a stack
        let stack_start = 0x00080000 + ::KERNEL_OFFSET;
        let stack_end = 0x0009F000 + ::KERNEL_OFFSET;

        // Initialize paging
        let (mut active_table, tcb_offset) = paging::init(0, stack_start, stack_end);

        // Set up GDT
        gdt::init(tcb_offset, stack_end);

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
        CPU_COUNT.store(1, Ordering::SeqCst);
        AP_READY.store(false, Ordering::SeqCst);
        BSP_READY.store(false, Ordering::SeqCst);

        // Setup kernel heap
        {
            // Map heap pages
            let heap_start_page = Page::containing_address(VirtualAddress::new(::KERNEL_HEAP_OFFSET));
            let heap_end_page = Page::containing_address(VirtualAddress::new(::KERNEL_HEAP_OFFSET + ::KERNEL_HEAP_SIZE-1));
            for page in Page::range_inclusive(heap_start_page, heap_end_page) {
                active_table.map(page, entry::PRESENT | entry::GLOBAL | entry::WRITABLE | entry::NO_EXECUTE);
            }

            // Init the allocator
            allocator::init(::KERNEL_HEAP_OFFSET, ::KERNEL_HEAP_SIZE);
        }

        // Initialize devices
        device::init(&mut active_table);

        // Read ACPI tables, starts APs
        acpi::init(&mut active_table);

        BSP_READY.store(true, Ordering::SeqCst);
    }

    kmain(CPU_COUNT.load(Ordering::SeqCst));
}

/// Entry to rust for an AP
pub unsafe extern fn kstart_ap(cpu_id: usize, bsp_table: usize, stack_start: usize, stack_end: usize) -> ! {
    {
        assert_eq!(BSS_TEST_ZERO, 0);
        assert_eq!(DATA_TEST_NONZERO, 0xFFFFFFFFFFFFFFFF);

        // Initialize paging
        let tcb_offset = paging::init_ap(cpu_id, bsp_table, stack_start, stack_end);

        // Set up GDT for AP
        gdt::init(tcb_offset, stack_end);

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

        // Initialize devices (for AP)
        device::init_ap();

        AP_READY.store(true, Ordering::SeqCst);
    }

    while ! BSP_READY.load(Ordering::SeqCst) {
        interrupt::pause();
    }

    kmain_ap(cpu_id);
}

pub unsafe fn usermode(ip: usize, sp: usize) -> ! {
    // Go to usermode
    asm!("xchg bx, bx
        mov ds, ax
        mov es, ax
        mov fs, bx
        mov gs, ax
        push rax
        push rcx
        push rdx
        push rsi
        push rdi
        iretq"
        : // No output because it never returns
        :   "{rax}"(gdt::GDT_USER_DATA << 3 | 3), // Data segment
            "{rbx}"(gdt::GDT_USER_TLS << 3 | 3), // TLS segment
            "{rcx}"(sp), // Stack pointer
            "{rdx}"(3 << 12 | 1 << 9), // Flags - Set IOPL and interrupt enable flag
            "{rsi}"(gdt::GDT_USER_CODE << 3 | 3), // Code segment
            "{rdi}"(ip) // IP
        : // No clobers because it never returns
        : "intel", "volatile");
    unreachable!();
}
