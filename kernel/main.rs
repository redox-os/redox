#![crate_name="kernel"]
#![crate_type="staticlib"]
#![feature(alloc)]
#![feature(allocator)]
#![feature(arc_counts)]
#![feature(augmented_assignments)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(core_str_ext)]
#![feature(core_slice_ext)]
#![feature(fnbox)]
#![feature(fundamental)]
#![feature(lang_items)]
#![feature(op_assign_traits)]
#![feature(unboxed_closures)]
#![feature(unsafe_no_drop_flag)]
#![feature(unwind_attributes)]
#![feature(vec_push_all)]
#![feature(zero_one)]
#![feature(collections_range)]
#![no_std]

#![allow(deprecated)]
#![deny(warnings)]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate collections;

extern crate system;

use acpi::Acpi;

use alloc::boxed::Box;

use arch::context::{context_switch, Context};
use arch::memory;
use arch::paging::Page;
use arch::regs::Regs;
use arch::tss::TSS;

use collections::string::ToString;

use core::{ptr, mem, usize};
use core::slice::SliceExt;

use common::time::Duration;

use drivers::pci;
use drivers::io::{Io, Pio};
use drivers::ps2::*;
use drivers::rtc::*;
use drivers::serial::*;

use env::Environment;

use schemes::context::*;
use schemes::debug::*;
use schemes::display::*;
use schemes::interrupt::*;
use schemes::memory::*;
use schemes::test::*;

use syscall::execute::execute;
use syscall::{do_sys_chdir, do_sys_exit, do_sys_open, syscall_handle};

pub use system::externs::*;

/// Common std-like functionality
#[macro_use]
pub mod common;
#[macro_use]
pub mod macros;
/// Allocation
pub mod alloc_system;
/// ACPI
pub mod acpi;
/// Architecture dependant
pub mod arch;
/// Audio
pub mod audio;
/// Disk drivers
pub mod disk;
/// Various drivers
pub mod drivers;
/// Environment
pub mod env;
/// Filesystems
pub mod fs;
/// Various graphical methods
pub mod graphics;
/// Networking
pub mod network;
/// Panic
pub mod panic;
/// Schemes
pub mod schemes;
/// Synchronization
pub mod sync;
/// System calls
pub mod syscall;
/// USB input/output
pub mod usb;

pub static mut TSS_PTR: Option<&'static mut TSS> = None;
pub static mut ENV_PTR: Option<&'static mut Environment> = None;

pub fn env() -> &'static Environment {
    unsafe {
        match ENV_PTR {
            Some(&mut ref p) => p,
            None => unreachable!(),
        }
    }
}

/// Pit duration
static PIT_DURATION: Duration = Duration {
    secs: 0,
    nanos: 4500572,
};

/// Idle loop (active while idle)
fn idle_loop() {
    loop {
        unsafe { asm!("cli" : : : : "intel", "volatile"); }

        env().on_poll();

        let mut halt = true;

        for context in env().contexts.lock().iter().skip(1) {
            if ! context.blocked {
                halt = false;
                break;
            }
        }

        if halt {
            unsafe { asm!("sti ; hlt" : : : : "intel", "volatile"); }
        } else {
            unsafe { asm!("sti ; nop" : : : : "intel", "volatile"); }
            unsafe { context_switch(); }
        }
    }
}

extern {
    static mut __text_start: u8;
    static mut __text_end: u8;
    static mut __rodata_start: u8;
    static mut __rodata_end: u8;
    static mut __data_start: u8;
    static mut __data_end: u8;
    static mut __bss_start: u8;
    static mut __bss_end: u8;
}

static BSS_TEST_ZERO: usize = 0;
static BSS_TEST_NONZERO: usize = usize::MAX;

/// Initialize kernel
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

        assert_eq!(BSS_TEST_ZERO, 0);
        assert_eq!(BSS_TEST_NONZERO, usize::MAX);
    }

    // Setup paging, this allows for memory allocation
    Page::init();
    memory::cluster_init();
    // Unmap first page to catch null pointer errors (after reading memory map)
    Page::new(0).unmap();

    //Remap text
    {
        let start_ptr = & __text_start as *const u8 as usize;
        let end_ptr = & __text_end as *const u8 as usize;
        if start_ptr as usize <= end_ptr {
            let size = end_ptr - start_ptr as usize;
            for page in 0..(size + 4095)/4096 {
                Page::new(start_ptr as usize + page * 4096).
                    map_kernel_read(start_ptr as usize + page * 4096);
            }
        }
    }

    //Remap rodata
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

    TSS_PTR = Some(&mut *(tss_data as *mut TSS));
    ENV_PTR = Some(&mut *Box::into_raw(Environment::new()));

    match ENV_PTR {
        Some(ref mut env) => {
            env.contexts.lock().push(Context::root());

            env.console.lock().draw = true;

            debugln!("Redox {} bits", mem::size_of::<usize>() * 8);

            if let Some(acpi) = Acpi::new() {
                env.schemes.lock().push(acpi);
            }

            *(env.clock_realtime.lock()) = Rtc::new().time();

            env.schemes.lock().push(Ps2::new());
            env.schemes.lock().push(Serial::new(0x3F8, 0x4));

            pci::pci_init(env);

            env.schemes.lock().push(DebugScheme::new());
            env.schemes.lock().push(box DisplayScheme);
            env.schemes.lock().push(box ContextScheme);
            env.schemes.lock().push(box InterruptScheme);
            env.schemes.lock().push(box MemoryScheme);
            env.schemes.lock().push(box TestScheme);

            env.contexts.lock().enabled = true;

            Context::spawn("kinit".to_string(),
            box move || {
                {
                    let wd_c = "file:/\0";
                    do_sys_chdir(wd_c.as_ptr()).unwrap();

                    let stdio_c = "debug:\0";
                    do_sys_open(stdio_c.as_ptr(), 0).unwrap();
                    do_sys_open(stdio_c.as_ptr(), 0).unwrap();
                    do_sys_open(stdio_c.as_ptr(), 0).unwrap();
                }

                if let Err(err) = execute(vec!["init".to_string()]) {
                    debugln!("INIT: Failed to execute: {}", err);
                }
            });
        },
        None => unreachable!(),
    }
}

#[cold]
#[inline(never)]
#[no_mangle]
/// Take regs for kernel calls and exceptions
pub extern "cdecl" fn kernel(interrupt: usize, mut regs: &mut Regs) {
    macro_rules! exception_inner {
        ($name:expr) => ({
            {
                let contexts = ::env().contexts.lock();
                if let Ok(context) = contexts.current() {
                    debugln!("PID {}: {}", context.pid, context.name);
                }
            }

            debugln!("  INT {:X}: {}", interrupt, $name);
            debugln!("    CS:  {:08X}    IP:  {:08X}    FLG: {:08X}", regs.cs, regs.ip, regs.flags);
            debugln!("    SS:  {:08X}    SP:  {:08X}    BP:  {:08X}", regs.ss, regs.sp, regs.bp);
            debugln!("    AX:  {:08X}    BX:  {:08X}    CX:  {:08X}    DX:  {:08X}", regs.ax, regs.bx, regs.cx, regs.dx);
            debugln!("    DI:  {:08X}    SI:  {:08X}", regs.di, regs.di);

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
            debugln!("    CR0: {:08X}    CR2: {:08X}    CR3: {:08X}    CR4: {:08X}", cr0, cr2, cr3, cr4);

            let mut fsw: usize = 0;
            let mut fcw: usize = 0;
            unsafe {
                asm!("fnstsw $0" : "=*m"(&mut fsw) : : : "intel", "volatile");
                asm!("fnstcw $0" : "=*m"(&mut fcw) : : : "intel", "volatile");
            }
            debugln!("    FSW: {:08X}    FCW: {:08X}", fsw, fcw);

            let sp = regs.sp as *const u32;
            for y in -15..16 {
                debug!("    {:>3}:", y * 8 * 4);
                for x in 0..8 {
                    debug!(" {:08X}", unsafe { ptr::read(sp.offset(-(x + y * 8))) });
                }
                debug!("\n");
            }
        })
    };

    macro_rules! exception {
        ($name:expr) => ({
            exception_inner!($name);

            loop {
                do_sys_exit(usize::MAX);
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
            debugln!("    ERR: {:08X}", error);

            loop {
                do_sys_exit(usize::MAX);
            }
        })
    };

    //Do not catch init interrupt
    if interrupt < 0xFF {
        env().interrupts.lock()[interrupt as usize] += 1;
    }

    match interrupt {
        0x20 => {
            {
                let mut clock_monotonic = env().clock_monotonic.lock();
                *clock_monotonic = *clock_monotonic + PIT_DURATION;
            }
            {
                let mut clock_realtime = env().clock_realtime.lock();
                *clock_realtime = *clock_realtime + PIT_DURATION;
            }

            if let Ok(mut current) = env().contexts.lock().current_mut() {
                current.time += 1;
            }

            unsafe { context_switch(); }
        }
        i @ 0x21 ... 0x2F => env().on_irq(i as u8 - 0x20),
        0x80 => syscall_handle(regs),
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
