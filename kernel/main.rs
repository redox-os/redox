#![crate_type="staticlib"]
#![feature(alloc)]
#![feature(allocator)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(collections)]
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
use core::slice::{self, SliceExt};
use core::str;

use common::debug;
use common::event::{self, Event, EventOption};
use common::get_slice::GetSlice;
use common::memory;
use common::paging::Page;
use common::queue::Queue;
//use common::prompt;
use common::time::Duration;

use drivers::pci::*;
use drivers::pio::*;
use drivers::ps2::*;
use drivers::rtc::*;
use drivers::serial::*;

pub use externs::*;

use graphics::display::{self, Display};
use graphics::point::Point;

use programs::executor::execute;
use programs::scheme::*;
use programs::session::*;

use scheduler::context::*;

use schemes::Url;
use schemes::arp::*;
use schemes::context::*;
use schemes::debug::*;
use schemes::ethernet::*;
use schemes::icmp::*;
use schemes::ip::*;
use schemes::memory::*;
use schemes::display::*;

use syscall::common::{Regs, SYS_YIELD};
use syscall::handle::*;

/// Allocation
pub mod alloc_system;
/// Audio
pub mod audio;
/// Common std-like functionality
#[macro_use]
pub mod common;
/// Various drivers
// TODO: Move out of kernel space (like other microkernels)
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
/// Scheduling
pub mod scheduler;
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

pub unsafe fn kernel_events() {
    let session = &mut *session_ptr;
    let events = &mut *events_ptr;

    //asm!("cli");

    session.on_poll();

    loop {
        let event_option = events.pop();

        match event_option {
            Some(event) => {
                if debug_draw {
                    match event.to_option() {
                        EventOption::Key(key_event) => {
                            if key_event.pressed {
                                match key_event.scancode {
                                    event::K_BKSP => if !session.cmd.is_empty() {
                                        debug::db(8);
                                        session.cmd.pop();
                                    },
                                    _ => match key_event.character {
                                        '\0' => (),
                                        '\n' => {
                                            *::debug_command = session.cmd.clone() + "\n";

                                            debug::dl();
                                            session.cmd.clear();
                                        },
                                        _ => {
                                            session.cmd.push(key_event.character);
                                            debug::dc(key_event.character);
                                        },
                                    },
                                }
                            }
                        },
                        _ => (),
                    }
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

    /*
    if halt {
        asm!("sti");
        asm!("hlt");
    } else {
        asm!("sti");
    }
    */
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

pub unsafe extern "cdecl" fn scheme_loop() {
    let session = &mut *session_ptr;

    loop {

    }
}

/// Initialize kernel
unsafe fn init(font_data: usize) {
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

    debug!("Redox ");
    debug::dd(mem::size_of::<usize>() * 8);
    debug!(" bits");
    debug::dl();

    clock_realtime = Rtc::new().time();

    contexts_ptr = Box::into_raw(box Vec::new());

    session_ptr = Box::into_raw(Session::new());

    events_ptr = Box::into_raw(box Queue::new());

    let session = &mut *session_ptr;

    session.items.push(Ps2::new());
    session.items.push(Serial::new(0x3F8, 0x4));

    pci_init(session);

    session.items.push(box DebugScheme);
    session.items.push(box ContextScheme);
    session.items.push(box MemoryScheme);
    //session.items.push(box RandomScheme);
    //session.items.push(box TimeScheme);

    session.items.push(box EthernetScheme);
    session.items.push(box ArpScheme);
    session.items.push(box IcmpScheme);
    session.items.push(box IpScheme {
        arp: Vec::new()
    });
    //session.items.push(box DisplayScheme);

    /*
    Context::spawn(box move || {
        ArpScheme::reply_loop();
    });
    Context::spawn(box move || {
        IcmpScheme::reply_loop();
    });
    */

    (*contexts_ptr).push(Context::new(scheme_loop as usize, &Vec::new()));

    //debugln!("Enabling context switching");
    //debug_draw = false;
    context_enabled = true;

    if let Some(mut resource) = Url::from_str("file:/schemes/").open() {
        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);

        for folder in String::from_utf8_unchecked(vec).lines() {
            if folder.ends_with('/') {
                let scheme_item = SchemeItem::from_url(&Url::from_string("file:/schemes/".to_string() + &folder));

                let reenable = scheduler::start_no_ints();
                session.items.push(scheme_item);
                scheduler::end_no_ints(reenable);
            }
        }
    }

    {
        let path_string = "file:/apps/terminal/terminal.bin";
        let path = Url::from_string(path_string.to_string());
        let wd = Url::from_string(path_string.get_slice(None, Some(path_string.rfind('/').unwrap_or(0) + 1)).to_string());
        if let Some(context) = execute(&path, &wd, Vec::new()) {
            (*contexts_ptr).push(context);
        }
    }
}

fn dr(reg: &str, value: usize) {
    debug!("{}", reg);
    debug!(": ");
    debug::dh(value as usize);
    debug::dl();
}

#[cold]
#[inline(never)]
#[no_mangle]
/// Take regs for kernel calls and exceptions
pub unsafe extern "cdecl" fn kernel(interrupt: usize, mut regs: &mut Regs) {
    macro_rules! exception {
        ($name:expr) => ({
            debug!("{}", $name);
            debug::dl();

            dr("INT", interrupt);
            dr("CONTEXT", context_i);
            dr("CS", regs.cs);
            dr("IP", regs.ip);
            dr("FLAGS", regs.flags);
            dr("SS", regs.ss);
            dr("SP", regs.sp);
            dr("BP", regs.bp);
            dr("AX", regs.ax);
            dr("BX", regs.bx);
            dr("CX", regs.cx);
            dr("DX", regs.dx);
            dr("DI", regs.di);
            dr("SI", regs.si);

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

            context_exit(regs);
            loop {
                asm!("cli");
                asm!("hlt");
            }
        })
    };

    macro_rules! exception_error {
        ($name:expr) => ({
            debug!("{}", $name);
            debug::dl();

            dr("INT", interrupt);
            dr("CONTEXT", context_i);
            dr("ERROR", regs.ip);
            dr("CS", regs.flags);
            dr("IP", regs.cs);
            dr("FLAGS", regs.sp);
            //dr("SS", regs.error);
            dr("SP", regs.ss);
            dr("BP", regs.bp);
            dr("AX", regs.ax);
            dr("BX", regs.bx);
            dr("CX", regs.cx);
            dr("DX", regs.dx);
            dr("DI", regs.di);
            dr("SI", regs.si);

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

            context_exit(regs);
            loop {
                asm!("cli");
                asm!("hlt");
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

            context_switch(regs, true);
        },
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
        0x80 => syscall_handle(regs),
        0xFF => {
            init(regs.ax);
            if let Some(mut next) = (*contexts_ptr).get_mut(context_i) {
                next.map();
                next.restore(regs);
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
