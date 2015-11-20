#![crate_type="staticlib"]
#![feature(alloc)]
#![feature(allocator)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(core_simd)]
#![feature(core_str_ext)]
#![feature(core_slice_ext)]
#![feature(fnbox)]
#![feature(fundamental)]
#![feature(lang_items)]
#![feature(no_std)]
#![feature(unboxed_closures)]
#![feature(unsafe_no_drop_flag)]
#![feature(unwind_attributes)]
#![feature(vec_push_all)]
#![feature(raw)]
#![no_std]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate collections;

use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;

use core::mem;
use core::slice::SliceExt;

use common::debug;
use common::event::{self, Event, EventOption};
use common::get_slice::GetSlice;
use common::memory;
use common::paging::Page;
use common::queue::Queue;
// use common::prompt;
use common::time::Duration;

use drivers::pci::*;
use drivers::pio::*;
use drivers::ps2::*;
use drivers::rtc::*;
use drivers::serial::*;

use env::console::Console;

pub use externs::*;

use graphics::display;

use programs::executor::execute;
use programs::scheme::*;
use programs::session::*;

use scheduler::{Context, Regs, TSS};
use scheduler::context::{context_enabled, context_exit, context_switch, context_i, contexts_ptr};

use schemes::Url;
use schemes::arp::*;
use schemes::context::*;
use schemes::debug::*;
use schemes::ethernet::*;
use schemes::icmp::*;
use schemes::ip::*;
use schemes::memory::*;
// use schemes::display::*;

use syscall::handle::*;

/// Common std-like functionality
#[macro_use]
pub mod common;
/// Allocation
pub mod alloc_system;
/// Audio
pub mod audio;
/// Various drivers
/// TODO: Move out of kernel space (like other microkernels)
pub mod drivers;
/// Environment
pub mod env;
/// Externs
pub mod externs;
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
/// System calls
pub mod syscall;
/// USB input/output
pub mod usb;

pub static mut tss_ptr: *mut TSS = 0 as *mut TSS;

/// Default console for debugging
static mut console: *mut Console = 0 as *mut Console;

/// Clock realtime (default)
static mut clock_realtime: Duration = Duration {
    secs: 0,
    nanos: 0,
};

/// Monotonic clock
static mut clock_monotonic: Duration = Duration {
    secs: 0,
    nanos: 0,
};

/// Pit duration
static PIT_DURATION: Duration = Duration {
    secs: 0,
    nanos: 2250286,
};

/// Session pointer
static mut session_ptr: *mut Session = 0 as *mut Session;

/// Event pointer
static mut events_ptr: *mut Queue<Event> = 0 as *mut Queue<Event>;

/// Idle loop (active while idle)
unsafe fn idle_loop() -> ! {
    loop {
        asm!("cli" : : : : "intel", "volatile");

        let mut halt = true;

        let contexts = &*contexts_ptr;
        for i in 1..contexts.len() {
            match contexts.get(i) {
                Some(context) => if context.interrupted {
                    halt = false;
                    break;
                },
                None => (),
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
unsafe fn poll_loop() -> ! {
    let session = &mut *session_ptr;

    loop {
        session.on_poll();

        context_switch(false);
    }
}

/// Event loop
unsafe fn event_loop() -> ! {
    let events = &mut *events_ptr;
    let mut cmd = String::new();
    loop {
        loop {
            let reenable = scheduler::start_no_ints();

            let event_option = events.pop();

            scheduler::end_no_ints(reenable);

            match event_option {
                Some(event) => {
                    if (*console).draw {
                        match event.to_option() {
                            EventOption::Key(key_event) => {
                                if key_event.pressed {
                                    match key_event.scancode {
                                        event::K_F2 => {
                                            (*console).draw = false;
                                        }
                                        event::K_BKSP => if !cmd.is_empty() {
                                            debug::db(8);
                                            cmd.pop();
                                        },
                                        _ => match key_event.character {
                                            '\0' => (),
                                            '\n' => {
                                                let reenable = scheduler::start_no_ints();
                                                (*console).command = Some(cmd.clone());
                                                scheduler::end_no_ints(reenable);

                                                cmd.clear();
                                                debug::dl();
                                            }
                                            _ => {
                                                cmd.push(key_event.character);
                                                debug::dc(key_event.character);
                                            }
                                        },
                                    }
                                }
                            }
                            _ => (),
                        }
                    } else {
                        if event.code == 'k' && event.b as u8 == event::K_F1 && event.c > 0 {
                            (*console).draw = true;
                            (*console).redraw = true;
                        } else {
                            // TODO: Magical orbital hack
                            let reenable = scheduler::start_no_ints();
                            for item in (*::session_ptr).items.iter_mut() {
                                if item.scheme() == "orbital" {
                                    item.event(&event);
                                    break;
                                }
                            }
                            scheduler::end_no_ints(reenable);
                        }
                    }
                }
                None => break,
            }
        }

        if (*console).draw {
            if (*console).redraw {
                (*console).redraw = false;
                (*console).display.flip();
            }
        } else {
            // session.redraw();
        }

        context_switch(false);
    }
}

/// Initialize debug
pub unsafe fn debug_init() {
    Pio8::new(0x3F8 + 1).write(0x00);
    Pio8::new(0x3F8 + 3).write(0x80);
    Pio8::new(0x3F8 + 0).write(0x03);
    Pio8::new(0x3F8 + 1).write(0x00);
    Pio8::new(0x3F8 + 3).write(0x03);
    Pio8::new(0x3F8 + 2).write(0xC7);
    Pio8::new(0x3F8 + 4).write(0x0B);
    Pio8::new(0x3F8 + 1).write(0x01);
}

/// Initialize kernel
unsafe fn init(font_data: usize, tss_data: usize) {
    debugln!("INITIALIZING...");
    display::fonts = font_data;
    tss_ptr = tss_data as *mut TSS;

    console = 0 as *mut Console;

    clock_realtime.secs = 0;
    clock_realtime.nanos = 0;

    clock_monotonic.secs = 0;
    clock_monotonic.nanos = 0;

    contexts_ptr = 0 as *mut Vec<Box<Context>>;
    context_i = 0;
    context_enabled = false;

    session_ptr = 0 as *mut Session;

    events_ptr = 0 as *mut Queue<Event>;

    debug_init();

    Page::init();
    debugln!("Initializing memory");
    memory::memory_init();
    debugln!("Memory initialized");
    // Unmap first page to catch null pointer errors (after reading memory map)
    Page::new(0).unmap();

    console = Box::into_raw(Console::new());
    (*console).draw = true;

    debug!("Redox ");
    debug::dd(mem::size_of::<usize>() * 8);
    debug!(" bits");
    debug::dl();

    clock_realtime = Rtc::new().time();

    contexts_ptr = Box::into_raw(box Vec::new());
    (*contexts_ptr).push(Context::root());

    session_ptr = Box::into_raw(Session::new());

    events_ptr = Box::into_raw(box Queue::new());

    let session = &mut *session_ptr;

    session.items.push(Ps2::new());
    session.items.push(Serial::new(0x3F8, 0x4));

    pci_init(session);

    session.items.push(DebugScheme::new());
    session.items.push(box ContextScheme);
    session.items.push(box MemoryScheme);
    // session.items.push(box RandomScheme);
    // session.items.push(box TimeScheme);

    session.items.push(box EthernetScheme);
    session.items.push(box ArpScheme);
    session.items.push(box IcmpScheme);
    session.items.push(box IpScheme { arp: Vec::new() });
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

    context_enabled = true;

    // TODO: Run schemes in contexts
    if let Some(mut resource) = Url::from_str("file:/schemes/").open() {
        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);

        for folder in String::from_utf8_unchecked(vec).lines() {
            if folder.ends_with('/') {
                let scheme_item = SchemeItem::from_url(&Url::from_string("file:/schemes/"
                                                                             .to_string() +
                                                                         &folder));

                let reenable = scheduler::start_no_ints();
                session.items.push(scheme_item);
                scheduler::end_no_ints(reenable);
            }
        }
    }

    {
        let path_string = "file:/apps/shell/main.bin";
        let path = Url::from_string(path_string.to_string());
        let wd = Url::from_string(path_string.get_slice(None,
                                                        Some(path_string.rfind('/').unwrap_or(0) +
                                                             1))
                                             .to_string());
        execute(&path, &wd, Vec::new());
    }
}

#[cold]
#[inline(never)]
#[no_mangle]
/// Take regs for kernel calls and exceptions
pub unsafe extern "cdecl" fn kernel(interrupt: usize, mut regs: &mut Regs) {
    macro_rules! exception_inner {
        ($name:expr) => ({
            if let Some(context) = Context::current() {
                debugln!("PID {}: {}", context_i, context.name);
            } else {
                debugln!("PID {}", context_i,);
            }

            debugln!("  INT {:X}: {}", interrupt, $name);
            debugln!("    CS:  {:08X}    IP:  {:08X}    FLG: {:08X}", regs.cs, regs.ip, regs.flags);
            debugln!("    SS:  {:08X}    SP:  {:08X}    BP:  {:08X}", regs.ss, regs.sp, regs.bp);
            debugln!("    AX:  {:08X}    BX:  {:08X}    CX:  {:08X}    DX:  {:08X}", regs.ax, regs.bx, regs.cx, regs.dx);
            debugln!("    DI:  {:08X}    SI:  {:08X}", regs.di, regs.di);

            let cr0: usize;
            asm!("mov $0, cr0" : "=r"(cr0) : : : "intel", "volatile");

            let cr2: usize;
            asm!("mov $0, cr2" : "=r"(cr2) : : : "intel", "volatile");

            let cr3: usize;
            asm!("mov $0, cr3" : "=r"(cr3) : : : "intel", "volatile");

            let cr4: usize;
            asm!("mov $0, cr4" : "=r"(cr4) : : : "intel", "volatile");
            debugln!("    CR0: {:08X}    CR2: {:08X}    CR3: {:08X}    CR4: {:08X}", cr0, cr2, cr3, cr4);
        })
    };

    macro_rules! exception {
        ($name:expr) => ({
            exception_inner!($name);

            loop {
                context_exit();
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
                context_exit();
            }
        })
    };

    if interrupt >= 0x20 && interrupt < 0x30 {
        if interrupt >= 0x28 {
            Pio8::new(0xA0).write(0x20);
        }

        Pio8::new(0x20).write(0x20);
    }

    match interrupt {
        0x20 => {
            let reenable = scheduler::start_no_ints();
            clock_realtime = clock_realtime + PIT_DURATION;
            clock_monotonic = clock_monotonic + PIT_DURATION;
            scheduler::end_no_ints(reenable);

            context_switch(true);
        }
        0x21 => (*session_ptr).on_irq(0x1), // keyboard
        0x23 => (*session_ptr).on_irq(0x3), // serial 2 and 4
        0x24 => (*session_ptr).on_irq(0x4), // serial 1 and 3
        0x25 => (*session_ptr).on_irq(0x5), //parallel 2
        0x26 => (*session_ptr).on_irq(0x6), //floppy
        0x27 => (*session_ptr).on_irq(0x7), //parallel 1 or spurious
        0x28 => (*session_ptr).on_irq(0x8), //RTC
        0x29 => (*session_ptr).on_irq(0x9), //pci
        0x2A => (*session_ptr).on_irq(0xA), //pci
        0x2B => (*session_ptr).on_irq(0xB), //pci
        0x2C => (*session_ptr).on_irq(0xC), //mouse
        0x2D => (*session_ptr).on_irq(0xD), //coprocessor
        0x2E => (*session_ptr).on_irq(0xE), //disk
        0x2F => (*session_ptr).on_irq(0xF), //disk
        0x80 => if !syscall_handle(regs) {
            exception!("Unknown Syscall");
        },
        0xFF => {
            init(regs.ax, regs.bx);
            idle_loop();
        }
        0x0 => exception!("Divide by zero exception"),
        0x1 => exception!("Debug exception"),
        0x2 => exception!("Non-maskable interrupt"),
        0x3 => exception!("Breakpoint exception"),
        0x4 => exception!("Overflow exception"),
        0x5 => exception!("Bound range exceeded exception"),
        0x6 => exception!("Invalid opcode exception"),
        0x7 => exception!("Device not available exception"),
        0x8 => exception_error!("Double fault"),
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
