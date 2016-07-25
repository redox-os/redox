#![crate_name="kernel"]
#![crate_type="staticlib"]
#![feature(alloc, allocator, arc_counts, asm, box_syntax, collections, const_fn, core_intrinsics,
           fnbox, fundamental, lang_items, naked_functions, unboxed_closures, unsafe_no_drop_flag,
           unwind_attributes, collections_range, question_mark, type_ascription)]
#![no_std]

#![deny(warnings)]
// #![deny(missing_docs)]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate collections;

extern crate system;

use acpi::Acpi;

use alloc::boxed::Box;

use arch::context::{context_switch, Context};
use arch::memory;
use arch::paging::Page;
use arch::regs::Regs;
use arch::tss::Tss;

use collections::{String, Vec};
use collections::string::ToString;

use core::{mem, usize};

use common::time::Duration;

use drivers::pci;
use drivers::io::{Io, Pio};
use drivers::ps2::*;
use drivers::rtc::*;
use drivers::serial::{self, Serial};

use env::Environment;

use graphics::display;

use network::schemes::{ArpScheme, EthernetScheme, IcmpScheme, IpScheme, TcpScheme, UdpScheme};

use schemes::context::ContextScheme;
use schemes::debug::DebugScheme;
use schemes::disk::DiskScheme;
use schemes::display::DisplayScheme;
use schemes::env::EnvScheme;
use schemes::initfs::InitFsScheme;
use schemes::interrupt::InterruptScheme;
use schemes::memory::MemoryScheme;
use schemes::syslog::SyslogScheme;
use schemes::test::TestScheme;

use syscall::process::exit;
use syscall::execute::execute;

pub use externs::*;

/// Common std-like functionality.
///
/// This module implements basic primitives for kernel space. They are not exposed to userspace.
#[macro_use]
pub mod common;
/// Logging.
///
/// This module contains the `syslog` function and the different log levels.
#[macro_use]
pub mod logging;
/// Macros used in the kernel.
#[macro_use]
pub mod macros;
/// Allocation lang items.
///
/// This module defines __rust_allocate lang item and friends, simply wrapping the allocation
/// method defined in `arch::memory`.
pub mod alloc_system;
/// ACPI implementation.
///
/// ACPI (Advanced Configuration and Power Interface) is the open standard for hardware detection,
/// power management, and hardware configuration. This module contains support for a subset of the
/// ACPI standard.
pub mod acpi;
/// Architecture dependent objects.
///
/// This module contains various mechanisms and primitives, such as ELF loading, interrupt locking,
/// memory paging, and so on.
///
/// This module is highly central to the kernel.
pub mod arch;
/// Audio drivers.
///
/// Drivers for controlling, playing, and configuring audio output.
///
/// This module contains `ac97` and `intelhda` audio drivers. These are likely to be moved to
/// userspace in the future.
pub mod audio;
/// Disk drivers.
///
/// Drivers for reading and writing disks. Currently includes drivers for following interfaces:
/// AHCI (Advanced Host Controller Interface), IDE (Integrated Drive Electronics), and ATA-1 (AT
/// Attachment Interface for Disk Drives).
pub mod disk;
/// Miscellaneous drivers.
///
/// This module contains miscellaneous drivers, including PCI (Peripheral Component
/// Interconnect), PS/2 (for keyboards and mice), RTC (real-time clock), SATA (Serial AT
/// Attachment), and keyboard layouts.
pub mod drivers;
/// The kernel environment.
///
/// This module defines the `Environment` struct, which has the job to track the state of the
/// kernel. This includes context management, system console, scheme registrar, and so on.
pub mod env;
/// Functions required for the Rust runtime, like memcpy and memset
pub mod externs;
/// File system.
///
/// This module manages virtual and non-virtual file systems. Furthermore, it defines URL,
/// `Scheme`, and `Resource`.
pub mod fs;
/// Graphic management.
///
/// This module contains the initial display manager and various graphics primitives.
pub mod graphics;
/// Networking.
///
/// This module contains drivers (e.g, intel8254x and rtl8139), primitives, schemes, and data
/// structures related to networking, providing Redox's networking stack.
pub mod network;
/// Kernel panic handling.
///
/// This module defines the kernel panic mechanism, which will halt the kernel (i.e. `sti; hlt;`)
/// in case of panics.
pub mod panic;
/// Schemes.
///
/// This module contains various schemes, such as `display:`, `debug:`, `memory:` and so on.
pub mod schemes;
/// Synchronization primitives.
///
/// This module provides various primitives for performing synchronization to avoid data races,
/// interrupts, and other unsafe conditions, when performing concurrent computation.
pub mod sync;
/// System calls and system call handler.
///
/// This module defines the system call handler and system calls of Redox.
///
/// System calls are the only way an userspace application can communicate with the kernel space.
/// Redox do, by design, have a very small number of syscalls when compared to Linux.
///
/// The system call interface is very similar to POSIX's system calls, making Redox able to run
/// many Unix programs.
pub mod syscall;
/// Drivers and primitives for USB input/output.
///
/// USB (Universal Serial Bus) is a standardized serial bus interface, used for many peripherals.
/// This modules contains drivers and other tools for USB.
pub mod usb;

/// The TTS pointer.
///
/// This static contains a mutable pointer to the TSS (task state segment), which is a data
/// structure used on x86-based architectures for holding information about a specific task. See
/// `Tss` for more information.
pub static mut TSS_PTR: Option<&'static mut Tss> = None;
/// The environment pointer.
///
/// The pointer to the kernel environment, holding the state of the kernel.
pub static mut ENV_PTR: Option<&'static mut Environment> = None;

/// Get the environment pointer.
///
/// This is unsafe, due to reading of a mutable static variable.
pub fn env() -> &'static Environment {
    unsafe {
        match ENV_PTR {
            Some(&mut ref p) => p,
            None => unreachable!(),
        }
    }
}

/// The PIT (programmable interval timer) duration.
///
/// This duration defines the PIT interval, which is added to the monotonic clock and the real time
/// clock, when interrupt 0x20 is received.
static PIT_DURATION: Duration = Duration {
    secs: 0,
    nanos: 4500572,
};

/// The idle loop.
///
/// This loop runs while the system is idle.
fn idle_loop() {
    loop {
        unsafe { asm!("cli" : : : : "intel", "volatile"); }

        let mut halt = true;

        for context in unsafe { & *env().contexts.get() }.iter().skip(1) {
            if context.blocked == 0 {
                halt = false;
                break;
            }
        }

        if halt {
            unsafe { asm!("sti ; hlt" : : : : "intel", "volatile"); }
        } else {
            unsafe { asm!("sti ; nop ; cli" : : : : "intel", "volatile"); }
            unsafe { context_switch(); }
        }
    }
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

/// Test of zero values in BSS.
static BSS_TEST_ZERO: usize = 0;
/// Test of non-zero values in BSS.
static BSS_TEST_NONZERO: usize = !0;

/// Initialize the kernel.
///
/// This will initialize the kernel: the environment, the memory allocator, the memory pager, PCI and so
/// on.
///
/// Note that this will not start the event loop.
unsafe fn init(tss_data: usize) {

    // Test
    assume!(true);

    // Zero BSS, this initializes statics that are set to 0
    {
        let start_ptr = &mut __bss_start as *mut u8;
        let end_ptr = & __bss_end as *const u8 as usize;

        if start_ptr as usize <= end_ptr {
            let size = end_ptr - start_ptr as usize;
            memset(start_ptr, 0, size);
        }

        debug_assert_eq!(BSS_TEST_ZERO, 0);
        debug_assert_eq!(BSS_TEST_NONZERO, usize::MAX);
    }

    // Setup paging, this allows for memory allocation
    Page::init();
    memory::cluster_init();

    // Get the serial information
    serial::bda_init();

    // Get the VBE information before unmapping the first megabyte
    display::vbe_init();

    // Unmap first page (TODO: Unmap more)
    {
        let start_ptr = 0;
        let end_ptr = 0x1000;

        if start_ptr <= end_ptr {
            let size = end_ptr - start_ptr;
            for page in 0..(size + 4095)/4096 {
                Page::new(start_ptr + page * 4096).unmap();
            }
        }
    }

    // Remap text
    {
        let start_ptr = & __text_start as *const u8 as usize;
        let end_ptr = & __text_end as *const u8 as usize;
        if start_ptr <= end_ptr {
            let size = end_ptr - start_ptr;
            for page in 0..(size + 4095)/4096 {
                Page::new(start_ptr + page * 4096).
                    map_kernel_read(start_ptr + page * 4096);
            }
        }
    }

    // Remap rodata
    {
        let start_ptr = & __rodata_start as *const u8 as usize;
        let end_ptr = & __rodata_end as *const u8 as usize;
        if start_ptr <= end_ptr {
            let size = end_ptr - start_ptr;
            for page in 0..(size + 4095)/4096 {
                Page::new(start_ptr + page * 4096).
                    map_kernel_read(start_ptr + page * 4096);
            }
        }
    }

    // Unmap logical/high mem
    {
        let start_ptr = 0x80000000;
        let end_ptr = 0xE0000000;

        if start_ptr <= end_ptr {
            let size = end_ptr - start_ptr;
            for page in 0..(size + 4095)/4096 {
                Page::new(start_ptr + page * 4096).unmap();
            }
        }
    }

    TSS_PTR = Some(&mut *(tss_data as *mut Tss));
    ENV_PTR = Some(&mut *Box::into_raw(Environment::new()));

    match ENV_PTR {
        Some(ref mut env) => {
            (&mut *env.contexts.get()).push(Context::root());

            let mut serial = Serial::new(0x3F8, 0x4);

            let mut term_columns;
            let mut term_lines;
            match (& *::env().console.get()).display {
                Some(ref display) => {
                    term_columns = format!("{}", display.width/8);
                    term_lines = format!("{}", display.height/16);
                },
                None => {
                    term_columns = String::new();
                    term_lines = String::new();
                    //Magic for getting serial size
                    serial.write("ANSI Terminal Size:\n\x1B[s\x1B[9999;9999f\x1B[6n\x1B[u".as_bytes());
                    let mut escaped = 0;
                    let mut param = 0;
                    loop {
                        let c = serial.readb() as char;
                        match c {
                            '\0' => break,
                            '\x1B' if escaped == 0 => escaped = 1,
                            '[' if escaped == 1 => escaped = 2,
                            '0' ... '9' if escaped == 2 => if param == 0 {
                                term_lines.push(c);
                            } else if param == 1 {
                                term_columns.push(c);
                            },
                            ';' if escaped == 2 => param += 1,
                            'R' if escaped == 2 => break,
                            _ => {
                                escaped = 0;
                                param = 0;
                            }
                        }
                    }
                    serial.write(term_columns.as_bytes());
                    serial.write(", ".as_bytes());
                    serial.write(term_lines.as_bytes());
                    serial.write("\n".as_bytes());
                }
            }

            (&mut *env.schemes.get()).push(serial);

            (&mut *env.console.get()).draw = true;

            syslog_info!("\x1B[1mRedox {} bits\x1B[0m", mem::size_of::<usize>() * 8);
            syslog_info!("  * text={:X}:{:X} rodata={:X}:{:X}",
                    & __text_start as *const u8 as usize, & __text_end as *const u8 as usize,
                    & __rodata_start as *const u8 as usize, & __rodata_end as *const u8 as usize);
            syslog_info!("  * data={:X}:{:X} bss={:X}:{:X}",
                    & __data_start as *const u8 as usize, & __data_end as *const u8 as usize,
                    & __bss_start as *const u8 as usize, & __bss_end as *const u8 as usize);

            if let Some(acpi) = Acpi::new() {
                (&mut *env.schemes.get()).push(acpi);
            }

            *env.clock_realtime.get() = Rtc::new().time();

            (&mut *env.schemes.get()).push(Ps2::new());

            pci::pci_init(env);

            (&mut *env.schemes.get()).push(DebugScheme::new());
            (&mut *env.schemes.get()).push(InitFsScheme::new());
            (&mut *env.schemes.get()).push(box ContextScheme);
            (&mut *env.schemes.get()).push(box DisplayScheme);
            (&mut *env.schemes.get()).push(box EnvScheme);
            (&mut *env.schemes.get()).push(box InterruptScheme);
            (&mut *env.schemes.get()).push(box MemoryScheme);
            (&mut *env.schemes.get()).push(box SyslogScheme);
            (&mut *env.schemes.get()).push(box TestScheme);

            //TODO: Do not do this! Find a better way
            let mut disks = Vec::new();
            disks.append(&mut *env.disks.get());
            (&mut *env.schemes.get()).push(DiskScheme::new(disks));

            /*
            let mut nics = Vec::new();
            nics.append(&mut env.nics.lock());
            (&mut *env.schemes.get()).push(NetworkScheme::new(nics));
            */

            (&mut *env.schemes.get()).push(box EthernetScheme);
            //(&mut *env.schemes.get()).push(box ArpScheme);
            //(&mut *env.schemes.get()).push(box IcmpScheme);
            (&mut *env.schemes.get()).push(box IpScheme {
                arp: Vec::new()
            });
            (&mut *env.schemes.get()).push(box TcpScheme);
            (&mut *env.schemes.get()).push(box UdpScheme);

            Context::spawn("karp".into(),
                           box move || {
                               ArpScheme::reply_loop();
                           });

            Context::spawn("kicmp".into(),
                           box move || {
                               IcmpScheme::reply_loop();
                           });

            (&mut *env.contexts.get()).enabled = true;

            Context::spawn("kinit".into(),
                           box move || {
                {
                    let wd_c = "initfs:/\0";
                    syscall::fs::chdir(wd_c.as_ptr()).unwrap();

                    let stdio_c = "debug:\0";
                    syscall::fs::open(stdio_c.as_ptr(), 0).unwrap();
                    syscall::fs::open(stdio_c.as_ptr(), 0).unwrap();
                    syscall::fs::open(stdio_c.as_ptr(), 0).unwrap();

                    let mut contexts = &mut *::env().contexts.get();
                    let current = contexts.current_mut().unwrap();

                    current.set_env_var("PATH", "file:/bin").unwrap();
                    current.set_env_var("COLUMNS", &term_columns).unwrap();
                    current.set_env_var("LINES", &term_lines).unwrap();
                    current.set_env_var("TTY", "debug:").unwrap();
                }

                syslog_info!("The kernel has finished booting. Running /bin/init");
                if let Err(err) = execute(vec!["initfs:/bin/init".to_string()]) {
                    syslog_info!("kernel: init: failed to execute: {}", err);
                }
            });
        },
        None => unreachable!(),
    }
}

#[cold]
#[inline(never)]
#[no_mangle]
/// Interrupt and exception handling.
pub extern "cdecl" fn kernel(interrupt: usize, mut regs: &mut Regs) {
    macro_rules! exception_inner {
        ($name:expr) => ({
            {
                let contexts = unsafe { &mut *::env().contexts.get() };
                if let Ok(context) = contexts.current() {
                    syslog_info!("PID {}: {}", context.pid, context.name);

                    if let Some(current_syscall) = context.current_syscall {
                        syslog_info!("  SYS {:X}: {} {} {:X} {:X} {:X}", current_syscall.0, current_syscall.1, syscall::name(current_syscall.1), current_syscall.2, current_syscall.3, current_syscall.4);
                    }
                }
            }

            syslog_info!("  INT {:X}: {}", interrupt, $name);
            syslog_info!("    CS:  {:08X}    IP:  {:08X}    FLG: {:08X}", regs.cs, regs.ip, regs.flags);
            syslog_info!("    SS:  {:08X}    SP:  {:08X}    BP:  {:08X}", regs.ss, regs.sp, regs.bp);
            syslog_info!("    AX:  {:08X}    BX:  {:08X}    CX:  {:08X}    DX:  {:08X}", regs.ax, regs.bx, regs.cx, regs.dx);
            syslog_info!("    DI:  {:08X}    SI:  {:08X}", regs.di, regs.di);

            let cr0: usize;
            let cr2: usize;
            let cr3: usize;
            let cr4: usize;
            unsafe {
                asm!("mov $0, cr0" : "=r"(cr0) : : : "intel", "volatile");
                asm!("mov $0, cr2" : "=r"(cr2) : : : "intel", "volatile");
                asm!("mov $0, cr3" : "=r"(cr3) : : : "intel", "volatile");
                asm!("mov $0, cr4" : "=r"(cr4) : : : "intel", "volatile");
            }
            syslog_info!("    CR0: {:08X}    CR2: {:08X}    CR3: {:08X}    CR4: {:08X}", cr0, cr2, cr3, cr4);

            let mut fsw: usize = 0;
            let mut fcw: usize = 0;
            unsafe {
                asm!("fnstsw $0" : "=*m"(&mut fsw) : : : "intel", "volatile");
                asm!("fnstcw $0" : "=*m"(&mut fcw) : : : "intel", "volatile");
            }
            syslog_info!("    FSW: {:08X}    FCW: {:08X}", fsw, fcw);

            /* TODO: Stack dump
            {
                let contexts = unsafe { & *::env().contexts.get() };
                if let Ok(context) = contexts.current() {
                    let sp = regs.sp as *const usize;
                    for y in -15..16 {
                        debug!("    {:>3}:", y * 8 * 4);
                        for x in 0..8 {
                            let p = unsafe { sp.offset(-(x + y * 8)) };
                            if let Ok(_) = context.translate(p as usize, 1) {
                                debug!(" {:08X}", unsafe { ptr::read(p) });
                            } else if context.kernel_stack > 0 && (p as usize) >= context.kernel_stack && (p as usize) < context.kernel_stack + CONTEXT_STACK_SIZE {
                                debug!(" {:08X}", unsafe { ptr::read(p) });
                            } else {
                                debug!(" ????????");
                            }
                        }
                        debug!("\n");
                    }
                }
            }
            */
        })
    };

    macro_rules! exception {
        ($name:expr) => ({
            exception_inner!($name);

            loop {
                exit(127);
            }
        })
    };

    macro_rules! exception_error {
        ($name:expr) => ({
            let error = regs.ip;
            regs.ip = regs.cs;
            regs.cs = regs.flags;
            regs.flags = regs.sp;
            regs.sp = regs.ss;
            regs.ss = 0;
            //regs.ss = regs.error;

            exception_inner!($name);
            syslog_info!("    ERR: {:08X}", error);

            loop {
                exit(127);
            }
        })
    };

    // Do not catch init interrupt
    if interrupt < 0xFF {
        unsafe { (&mut *env().interrupts.get())[interrupt as usize] += 1 };
    }

    match interrupt {
        0x20 => {
            {
                let mut clock_monotonic = unsafe { &mut *env().clock_monotonic.get() };
                *clock_monotonic = *clock_monotonic + PIT_DURATION;
            }
            {
                let mut clock_realtime = unsafe { &mut *env().clock_realtime.get() };
                *clock_realtime = *clock_realtime + PIT_DURATION;
            }

            if let Ok(mut current) = unsafe { &mut *env().contexts.get() }.current_mut() {
                current.time += 1;
            }

            unsafe { context_switch(); }
        }
        i @ 0x21 ... 0x2F => {
            env().on_irq(i as u8 - 0x20);
        },
        0x80 => syscall::handle(regs),
        0xFF => {
            unsafe {
                init(regs.ax);
                idle_loop();
            }
        },
        0x0 => exception!("Divide by zero exception"),
        0x1 => exception!("Debug exception"),
        0x2 => exception!("Non-maskable interrupt"),
        0x3 => exception!("Breakpoint exception"),
        0x4 => exception!("Overflow exception"),
        0x5 => exception!("Bound range exceeded exception"),
        0x6 => exception!("Invalid opcode exception"),
        0x7 => exception!("Device not available exception"),
        0x8 => exception_error!("Double fault"),
        0x9 => exception!("Coprocessor Segment Overrun"), // legacy
        0xA => exception_error!("Invalid TSS exception"),
        0xB => exception_error!("Segment not present exception"),
        0xC => exception_error!("Stack-segment fault"),
        0xD => exception_error!("General protection fault"),
        0xE => exception_error!("Page fault"),
        0x10 => exception!("x87 floating-point exception"),
        0x11 => exception_error!("Alignment check exception"),
        0x12 => exception!("Machine check exception"),
        0x13 => exception!("SIMD floating-point exception"),
        0x14 => exception!("Virtualization exception"),
        0x1E => exception_error!("Security exception"),
        _ => exception!("Unknown Interrupt"),
    }

    if interrupt >= 0x20 && interrupt < 0x30 {
        if interrupt >= 0x28 {
            Pio::<u8>::new(0xA0).write(0x20);
        }

        Pio::<u8>::new(0x20).write(0x20);
    }
}
