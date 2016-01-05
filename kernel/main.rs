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
#![no_std]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate collections;

use acpi::Acpi;

use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;

use core::cell::UnsafeCell;
use core::{ptr, mem, usize};
use core::slice::SliceExt;

use common::event::{self, EVENT_KEY, EventOption};
use common::memory;
use common::paging::Page;
use common::time::Duration;

use drivers::pci;
use drivers::pio::*;
use drivers::ps2::*;
use drivers::rtc::*;
use drivers::serial::*;

use env::Environment;

pub use externs::*;

use graphics::display;

use programs::executor::execute;
use programs::scheme::*;

use scheduler::{Context, Regs, TSS};
use scheduler::context::context_switch;

use schemes::Url;
use schemes::arp::*;
use schemes::context::*;
use schemes::debug::*;
use schemes::ethernet::*;
use schemes::icmp::*;
use schemes::interrupt::*;
use schemes::ip::*;
use schemes::memory::*;
// use schemes::display::*;

use syscall::handle::*;

/// Common std-like functionality
#[macro_use]
pub mod common;
/// ACPI
pub mod acpi;
/// Allocation
pub mod alloc_system;
/// Audio
pub mod audio;
/// Disk drivers
pub mod disk;
/// Various drivers
pub mod drivers;
/// Environment
pub mod env;
/// Externs
pub mod externs;
/// Filesystems
pub mod fs;
/// Various graphical methods
pub mod graphics;
/// Network
pub mod network;
/// Panic
pub mod panic;
/// Programs
pub mod programs;
/// Schemes
pub mod schemes;
/// Scheduling
pub mod scheduler;
/// Sync primatives
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
    nanos: 2250286,
};

/// Idle loop (active while idle)
unsafe fn idle_loop() {
    loop {
        asm!("cli" : : : : "intel", "volatile");

        let mut halt = true;

        for i in env().contexts.lock().iter().skip(1) {
            if i.interrupted {
                halt = false;
                break;
            }
        }


        if halt {
            asm!("sti" : : : : "intel", "volatile");
            asm!("hlt" : : : : "intel", "volatile");
        } else {
            asm!("sti" : : : : "intel", "volatile");
        }


        context_switch(false);
    }
}

/// Event poll loop
fn poll_loop() {
    loop {
        env().on_poll();

        unsafe { context_switch(false) };
    }
}

/// Event loop
fn event_loop() {
    {
        let mut console = env().console.lock();
        console.instant = false;
    }

    let mut cmd = String::new();
    loop {
        loop {
            let mut console = env().console.lock();
            match env().events.lock().pop_front() {
                Some(event) => {
                    if console.draw {
                        match event.to_option() {
                            EventOption::Key(key_event) => {
                                if key_event.pressed {
                                    match key_event.scancode {
                                        event::K_F2 => {
                                            console.draw = false;
                                        }
                                        event::K_BKSP => if !cmd.is_empty() {
                                            console.write(&[8]);
                                            cmd.pop();
                                        },
                                        _ => match key_event.character {
                                            '\0' => (),
                                            '\n' => {
                                                console.command = Some(cmd.clone());

                                                cmd.clear();
                                                console.write(&[10]);
                                            }
                                            _ => {
                                                cmd.push(key_event.character);
                                                console.write(&[key_event.character as u8]);
                                            }
                                        },
                                    }
                                }
                            }
                            _ => (),
                        }
                    } else {
                        if event.code == EVENT_KEY && event.b as u8 == event::K_F1 && event.c > 0 {
                            console.draw = true;
                            console.redraw = true;
                        } else {
                            // TODO: Magical orbital hack
                            unsafe {
                                for scheme in env().schemes.iter() {
                                    if (*scheme.get()).scheme() == "orbital" {
                                        (*scheme.get()).event(&event);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                None => break,
            }
        }

        {
            let mut console = env().console.lock();
            console.instant = false;
            if console.draw && console.redraw {
                console.redraw = false;
                console.display.flip();
            }
        }

        unsafe { context_switch(false) };
    }
}

static BSS_TEST_ZERO: usize = 0;
static BSS_TEST_NONZERO: usize = usize::MAX;

/// Initialize kernel
unsafe fn init(font_data: usize, tss_data: usize) {
    // Zero BSS, this initializes statics that are set to 0
    {
        extern {
            static mut __bss_start: u8;
            static mut __bss_end: u8;
        }

        let start_ptr = &mut __bss_start;
        let end_ptr = &mut __bss_end;

        if start_ptr as *const _ as usize <= end_ptr as *const _ as usize {
            let size = end_ptr as *const _ as usize - start_ptr as *const _ as usize;
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

    display::fonts = font_data;
    TSS_PTR = Some(&mut *(tss_data as *mut TSS));
    ENV_PTR = Some(&mut *Box::into_raw(Environment::new()));

    match ENV_PTR {
        Some(ref mut env) => {
            env.contexts.lock().push(Context::root());
            env.console.lock().draw = true;

            debug!("Redox {} bits\n", mem::size_of::<usize>() * 8);

            if let Some(acpi) = Acpi::new() {
                env.schemes.push(UnsafeCell::new(acpi));
            }

            *(env.clock_realtime.lock()) = Rtc::new().time();

            env.schemes.push(UnsafeCell::new(Ps2::new()));
            env.schemes.push(UnsafeCell::new(Serial::new(0x3F8, 0x4)));

            pci::pci_init(env);

            env.schemes.push(UnsafeCell::new(DebugScheme::new()));
            env.schemes.push(UnsafeCell::new(box ContextScheme));
            env.schemes.push(UnsafeCell::new(box InterruptScheme));
            env.schemes.push(UnsafeCell::new(box MemoryScheme));
            // session.items.push(box RandomScheme);
            // session.items.push(box TimeScheme);

            env.schemes.push(UnsafeCell::new(box EthernetScheme));
            env.schemes.push(UnsafeCell::new(box ArpScheme));
            env.schemes.push(UnsafeCell::new(box IcmpScheme));
            env.schemes.push(UnsafeCell::new(box IpScheme { arp: Vec::new() }));
            // session.items.push(box DisplayScheme);

            Context::spawn("kpoll".to_string(),
            box move || {
                poll_loop();
            });

            Context::spawn("kevent".to_string(),
            box move || {
                event_loop();
            });

            Context::spawn("karp".to_string(),
            box move || {
                ArpScheme::reply_loop();
            });

            Context::spawn("kicmp".to_string(),
            box move || {
                IcmpScheme::reply_loop();
            });

            env.contexts.lock().enabled = true;

            if let Ok(mut resource) = Url::from_str("file:/schemes/").open() {
                let mut vec: Vec<u8> = Vec::new();
                let _ = resource.read_to_end(&mut vec);

                for folder in String::from_utf8_unchecked(vec).lines() {
                    if folder.ends_with('/') {
                        let scheme_item = SchemeItem::from_url(&Url::from_string("file:/schemes/"
                                                                                 .to_string() +
                                                                                 &folder));

                        env.schemes.push(UnsafeCell::new(scheme_item));
                    }
                }
            }

            Context::spawn("kinit".to_string(),
            box move || {
                {
                    let wd_c = "file:/\0";
                    do_sys_chdir(wd_c.as_ptr());

                    let stdio_c = "debug:\0";
                    do_sys_open(stdio_c.as_ptr(), 0);
                    do_sys_open(stdio_c.as_ptr(), 0);
                    do_sys_open(stdio_c.as_ptr(), 0);
                }

                execute(Url::from_str("file:/apps/login/main.bin"), Vec::new());
                debug!("INIT: Failed to execute\n");

                loop {
                    context_switch(false);
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
                if let Some(context) = contexts.current() {
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

    if interrupt >= 0x20 && interrupt < 0x30 {
        if interrupt >= 0x28 {
            unsafe { Pio8::new(0xA0).write(0x20) };
        }

        unsafe { Pio8::new(0x20).write(0x20) };
    }

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

            let switch = {
                let mut contexts = ::env().contexts.lock();
                if let Some(mut context) = contexts.current_mut() {
                    context.slices -= 1;
                    context.slice_total += 1;
                    context.slices == 0
                } else {
                    false
                }
            };

            if switch {
                unsafe { context_switch(true) };
            }
        }
        i @ 0x21 ... 0x2F => env().on_irq(i as u8 - 0x20),
        0x80 => if !syscall_handle(regs) {
            exception!("Unknown Syscall");
        },
        0xFF => {
            unsafe {
                init(regs.ax, regs.bx);
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
}
