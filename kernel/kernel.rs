#![crate_type="staticlib"]
#![feature(alloc)]
#![feature(allocator)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(core_intrinsics)]
#![feature(core_simd)]
#![feature(core_str_ext)]
#![feature(fnbox)]
#![feature(fundamental)]
#![feature(lang_items)]
#![feature(no_std)]
#![feature(unboxed_closures)]
#![feature(unsafe_no_drop_flag)]
#![feature(unwind_attributes)]
#![feature(vec_push_all)]
#![feature(slice_concat_ext)]
#![no_std]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate collections;

use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;

use core::{mem, ptr};

use common::context::*;
use common::debug;
use common::event::{self, Event, EventOption, DisplayEvent};
use common::memory;
use common::paging::Page;
use common::queue::Queue;
use schemes::URL;
use common::scheduler;
use common::time::Duration;
use common::parse_path::*;

use drivers::pci::*;
use drivers::pio::*;
use drivers::ps2::*;
use drivers::rtc::*;
use drivers::serial::*;

pub use externs::*;

use graphics::bmp::BMPFile;
use graphics::display::{self, Display};
use graphics::point::Point;

use programs::executor::*;
use programs::package::*;
use programs::scheme::*;
use programs::session::*;

use schemes::arp::*;
use schemes::context::*;
use schemes::debug::*;
use schemes::ethernet::*;
use schemes::ip::*;
use schemes::memory::*;
use schemes::random::*;
use schemes::time::*;
use schemes::display::*;
use schemes::events::*;

use syscall::handle::*;

/// Allocation
pub mod alloc_system;
/// Audio
pub mod audio;
/// Common std-like functionality
#[macro_use]
pub mod common;
/// Various drivers
pub mod drivers;
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
/// System calls
pub mod syscall;
/// USB input/output
pub mod usb;

/// Default display for debugging
static mut debug_display: *mut Display = 0 as *mut Display;
/// Default point for debugging
static mut debug_point: Point = Point { x: 0, y: 0 };
/// Draw debug
static mut debug_draw: bool = false;
/// Redraw debug
static mut debug_redraw: bool = false;
/// Debug command
static mut debug_command: *mut String = 0 as *mut String;

/// Clock realtime (default)
static mut clock_realtime: Duration = Duration {
    secs: 0,
    nanos: 0
};

/// Monotonic clock
static mut clock_monotonic: Duration = Duration {
    secs: 0,
    nanos: 0
};

/// Pit duration
static PIT_DURATION: Duration = Duration {
    secs: 0,
    nanos: 2250286
};

/// Session pointer
static mut session_ptr: *mut Session = 0 as *mut Session;

/// Event pointer
static mut events_ptr: *mut Queue<Event> = 0 as *mut Queue<Event>;

/// Idle loop (active while idle)
unsafe fn idle_loop() -> ! {
    loop {
        asm!("cli");

        let mut halt = true;

        let contexts = & *contexts_ptr;
        for i in 1..contexts.len() {
            match contexts.get(i) {
                Some(context) => if context.interrupted {
                    halt = false;
                    break;
                },
                None => ()
            }
        }

        if halt {
            asm!("sti");
            asm!("hlt");
        } else {
            asm!("sti");
        }

        context_switch(true);
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
    let session = &mut *session_ptr;
    let events = &mut *events_ptr;
    let mut cmd = String::new();
    loop {
        loop {
            let reenable = scheduler::start_no_ints();

            let event_option = events.pop();

            scheduler::end_no_ints(reenable);

            match event_option {
                Some(event) => {
                    match event.to_option() {
                        EventOption::Key(key_event) => {
                            if key_event.pressed {
                                if debug_draw {
                                    match key_event.scancode {
                                        event::K_F2 => {
                                            ::debug_draw = false;
                                            EventResource::add_event(DisplayEvent { 
                                                restricted: false
                                            }.to_event());
                                        },
                                        event::K_BKSP => if !cmd.is_empty() {
                                            debug::db(8);
                                            cmd.pop();
                                        },
                                        _ => match key_event.character {
                                            '\0' => (),
                                            '\n' => {
                                                let reenable = scheduler::start_no_ints();
                                                *::debug_command = cmd + "\n";
                                                scheduler::end_no_ints(reenable);
                                                cmd = String::new();
                                                debug::dl();
                                            },
                                            _ => {
                                                cmd.push(key_event.character);
                                                debug::dc(key_event.character);
                                            },
                                        },
                                    }
                                    
                                } else {
                                    match key_event.scancode {
                                        event::K_F1 => {
                                            ::debug_draw = true;
                                            ::debug_redraw = true;
                                            EventResource::add_event(DisplayEvent { 
                                                restricted: true
                                            }.to_event());
                                        },
                                        _ => EventResource::add_event(event),
                                    }
                                }
                            }
                        },
                        _ => EventResource::add_event(event),
                    }
                },
                None => break
            }
        }

        if debug_draw {
            let display = &*debug_display;
            if debug_redraw {
                debug_redraw = false;
                display.flip();
            }
        } 

        context_switch(false);
    }
}

/// Initialize debug
pub unsafe fn debug_init() {
    PIO8::new(0x3F8 + 1).write(0x00);
    PIO8::new(0x3F8 + 3).write(0x80);
    PIO8::new(0x3F8 + 0).write(0x03);
    PIO8::new(0x3F8 + 1).write(0x00);
    PIO8::new(0x3F8 + 3).write(0x03);
    PIO8::new(0x3F8 + 2).write(0xC7);
    PIO8::new(0x3F8 + 4).write(0x0B);
    PIO8::new(0x3F8 + 1).write(0x01);
}

/// Initialize kernel
unsafe fn init(font_data: usize) {
    scheduler::start_no_ints();

    debug_display = 0 as *mut Display;
    debug_point = Point { x: 0, y: 0 };
    debug_draw = false;
    debug_redraw = false;

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
    memory::cluster_init();
    //Unmap first page to catch null pointer errors (after reading memory map)
    Page::new(0).unmap();

    ptr::write(display::FONTS, font_data);

    debug_display = Box::into_raw(Display::root());

    debug_draw = true;

    debug_command = Box::into_raw(box String::new());

    debugln!("WELCOME TO REDOX!");

    debug::d("Redox ");
    debug::dd(mem::size_of::<usize>() * 8);
    debug::d(" bits ");
    debug::dl();

    clock_realtime = RTC::new().time();

    contexts_ptr = Box::into_raw(box Vec::new());
    (*contexts_ptr).push(Context::root());

    session_ptr = Box::into_raw(Session::new());

    events_ptr = Box::into_raw(box Queue::new());

    let session = &mut *session_ptr;

    session.items.push(PS2::new());
    session.items.push(Serial::new(0x3F8, 0x4));

    pci_init(session);

    session.items.push(box ContextScheme);
    session.items.push(box DebugScheme);
    session.items.push(box MemoryScheme);
    session.items.push(box RandomScheme);
    session.items.push(box TimeScheme);

    session.items.push(box EthernetScheme);
    session.items.push(box ARPScheme);
    session.items.push(box IPScheme {
        arp: Vec::new()
    });
    session.items.push(box DisplayScheme);
    EventScheme::init();
    session.items.push(box EventScheme);

    Context::spawn(box move || {
        poll_loop();
    });
    Context::spawn(box move || {
        event_loop();
    });
    Context::spawn(box move || {
        ARPScheme::reply_loop();
    });

    debug::d("Reenabling interrupts\n");

    //Start interrupts
    scheduler::end_no_ints(true);

    debug::d("Loading schemes\n");
    if let Some(mut resource) = URL::from_str("file:///schemes/").open() {
        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);

        for folder in String::from_utf8_unchecked(vec).lines() {
            if folder.ends_with('/') {
                let scheme_item = SchemeItem::from_url(&URL::from_string(&("file:///schemes/".to_string() + &folder)));

                let reenable = scheduler::start_no_ints();
                session.items.push(scheme_item);
                scheduler::end_no_ints(reenable);
            }
        }
    }

    debug::d("Loading apps\n");
    if let Some(mut resource) = URL::from_str("file:///apps/").open() {
        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);

        for folder in String::from_utf8_unchecked(vec).lines() {
            if folder.ends_with('/') {
                let package = Package::from_url(&URL::from_string(&("file:///apps/".to_string() + folder)));
                let reenable = scheduler::start_no_ints();
                session.packages.push(package);
                scheduler::end_no_ints(reenable);
            }
        }
    }

    debug::d("Enabling context switching\n");
    debug_draw = false;
    context_enabled = true;

    debug::d("Launching windowing session\n");
    let wm = Package::from_url(&URL::from_string(&("file:///apps/".to_string() + "orbital/")));
    execute(&wm.binary, &wm.url, Vec::new());
    // launch session
}

fn dr(reg: &str, value: usize) {
    debug::d(reg);
    debug::d(": ");
    debug::dh(value as usize);
    debug::dl();
}

#[cold]
#[inline(never)]
#[no_mangle]
#[cfg(target_arch = "x86")]
/// Take regs for kernel calls and exceptions
pub unsafe extern "cdecl" fn kernel(
                            interrupt: usize, mut ax: usize, bx: usize, cx: usize, dx: usize, di: usize, si: usize, bp: usize, sp: usize,
                            ip: usize, flags: usize, error: usize) -> usize {
    macro_rules! exception {
        ($name:expr) => ({
            debug::d($name);
            debug::dl();

            dr("INT", interrupt);
            dr("CONTEXT", context_i);
            dr("IP", ip);
            dr("FLAGS", flags);
            dr("AX", ax);
            dr("BX", bx);
            dr("CX", cx);
            dr("DX", dx);
            dr("DI", di);
            dr("SI", si);
            dr("BP", bp);
            dr("SP", sp);

            let cr0: usize;
            asm!("mov $0, cr0" : "=r"(cr0) : : : "intel", "volatile");
            dr("CR0", cr0);

            let cr2: usize;
            asm!("mov $0, cr2" : "=r"(cr2) : : : "intel", "volatile");
            dr("CR2", cr2);

            let cr3: usize;
            asm!("mov $0, cr3" : "=r"(cr3) : : : "intel", "volatile");
            dr("CR3", cr3);

            let cr4: usize;
            asm!("mov $0, cr4" : "=r"(cr4) : : : "intel", "volatile");
            dr("CR4", cr4);

            do_sys_exit(-1);
            loop {
                asm!("cli");
                asm!("hlt");
            }
        })
    };

    macro_rules! exception_error {
        ($name:expr) => ({
            debug::d($name);
            debug::dl();

            dr("INT", interrupt);
            dr("CONTEXT", context_i);
            dr("IP", flags);
            dr("FLAGS", error);
            dr("ERROR", ip);
            dr("AX", ax);
            dr("BX", bx);
            dr("CX", cx);
            dr("DX", dx);
            dr("DI", di);
            dr("SI", si);
            dr("BP", bp);
            dr("SP", sp);

            let cr0: usize;
            asm!("mov $0, cr0" : "=r"(cr0) : : : "intel", "volatile");
            dr("CR0", cr0);

            let cr2: usize;
            asm!("mov $0, cr2" : "=r"(cr2) : : : "intel", "volatile");
            dr("CR2", cr2);

            let cr3: usize;
            asm!("mov $0, cr3" : "=r"(cr3) : : : "intel", "volatile");
            dr("CR3", cr3);

            let cr4: usize;
            asm!("mov $0, cr4" : "=r"(cr4) : : : "intel", "volatile");
            dr("CR4", cr4);

            do_sys_exit(-1);
            loop {
                asm!("cli");
                asm!("hlt");
            }
        })
    };

    if interrupt >= 0x20 && interrupt < 0x30 {
        if interrupt >= 0x28 {
            PIO8::new(0xA0).write(0x20);
        }

        PIO8::new(0x20).write(0x20);
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
        0x80 => ax = syscall_handle(ax, bx, cx, dx),
        0xFF => {
            init(ax);
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

    ax
}

#[cold]
#[inline(never)]
#[no_mangle]
#[cfg(target_arch = "x86_64")]
/// Take regs for kernel calls and exceptions
pub unsafe extern "cdecl" fn kernel(
                        s1: usize, s2: usize, s3: usize, s4: usize, s5: usize, s6: usize, //TODO Remove thse scratch values by modifying interrupts-x86_64.asm
                        interrupt: usize, mut ax: usize, bx: usize, cx: usize, dx: usize, di: usize, si: usize,
                        r8: usize, r9: usize, r10: usize, r11: usize, r12: usize, r13: usize, r14: usize, r15: usize,
                        bp: usize, sp: usize, ip: usize, flags: usize, error: usize) -> usize {
    macro_rules! exception {
        ($name:expr) => ({
            debug::d($name);
            debug::dl();

            dr("INT", interrupt);
            dr("CONTEXT", context_i);
            dr("IP", ip);
            dr("FLAGS", flags);
            dr("AX", ax);
            dr("BX", bx);
            dr("CX", cx);
            dr("DX", dx);
            dr("DI", di);
            dr("SI", si);
            dr("R8", r8);
            dr("R9", r9);
            dr("R10", r10);
            dr("R11", r11);
            dr("R12", r12);
            dr("R13", r13);
            dr("R14", r14);
            dr("R15", r15);
            dr("BP", bp);
            dr("SP", sp);

            let cr0: usize;
            asm!("mov $0, cr0" : "=r"(cr0) : : : "intel", "volatile");
            dr("CR0", cr0);

            let cr2: usize;
            asm!("mov $0, cr2" : "=r"(cr2) : : : "intel", "volatile");
            dr("CR2", cr2);

            let cr3: usize;
            asm!("mov $0, cr3" : "=r"(cr3) : : : "intel", "volatile");
            dr("CR3", cr3);

            let cr4: usize;
            asm!("mov $0, cr4" : "=r"(cr4) : : : "intel", "volatile");
            dr("CR4", cr4);

            do_sys_exit(-1);
            loop {
                asm!("cli");
                asm!("hlt");
            }
        })
    };

    macro_rules! exception_error {
        ($name:expr) => ({
            debug::d($name);
            debug::dl();

            dr("INT", interrupt);
            dr("CONTEXT", context_i);
            dr("IP", flags);
            dr("FLAGS", error);
            dr("ERROR", ip);
            dr("AX", ax);
            dr("BX", bx);
            dr("CX", cx);
            dr("DX", dx);
            dr("DI", di);
            dr("SI", si);
            dr("BP", bp);
            dr("SP", sp);
            dr("R8", r8);
            dr("R9", r9);
            dr("R10", r10);
            dr("R11", r11);
            dr("R12", r12);
            dr("R13", r13);
            dr("R14", r14);
            dr("R15", r15);

            let cr0: usize;
            asm!("mov $0, cr0" : "=r"(cr0) : : : "intel", "volatile");
            dr("CR0", cr0);

            let cr2: usize;
            asm!("mov $0, cr2" : "=r"(cr2) : : : "intel", "volatile");
            dr("CR2", cr2);

            let cr3: usize;
            asm!("mov $0, cr3" : "=r"(cr3) : : : "intel", "volatile");
            dr("CR3", cr3);

            let cr4: usize;
            asm!("mov $0, cr4" : "=r"(cr4) : : : "intel", "volatile");
            dr("CR4", cr4);

            do_sys_exit(-1);
            loop {
                asm!("cli");
                asm!("hlt");
            }
        })
    };

    if interrupt >= 0x20 && interrupt < 0x30 {
        if interrupt >= 0x28 {
            PIO8::new(0xA0).write(0x20);
        }

        PIO8::new(0x20).write(0x20);
    }

    match interrupt {
        0x20 => {
            let reenable = scheduler::start_no_ints();
            clock_realtime = clock_realtime + PIT_DURATION;
            clock_monotonic = clock_monotonic + PIT_DURATION;
            scheduler::end_no_ints(reenable);

            context_switch(true);
        }
        0x21 => (*session_ptr).on_irq(0x1), //keyboard
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
        0x80 => ax = syscall_handle(ax, bx, cx, dx),
        0xFF => {
            init(ax);
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

    ax
}
