#![crate_type="staticlib"]
#![feature(alloc)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(core_intrinsics)]
#![feature(core_simd)]
#![feature(core_slice_ext)]
#![feature(core_str_ext)]
#![feature(fnbox)]
#![feature(fundamental)]
#![feature(lang_items)]
#![feature(no_std)]
#![feature(unboxed_closures)]
#![feature(unsafe_no_drop_flag)]
#![feature(unwind_attributes)]
#![no_std]

extern crate alloc;

use alloc::boxed::Box;

use core::{cmp, mem, ptr};

use common::context::*;
use common::debug;
use common::event::{self, Event, EventOption};
use common::memory;
use common::paging::*;
use common::queue::Queue;
use common::resource::URL;
use common::scheduler::*;
use common::string::{String, ToString};
use common::time::Duration;
use common::vec::Vec;

use drivers::disk::*;
use drivers::pci::*;
use drivers::pio::*;
use drivers::ps2::*;
use drivers::rtc::*;
use drivers::serial::*;

pub use externs::*;

use graphics::bmp::*;
use graphics::color::Color;
use graphics::display::{self, Display};
use graphics::point::Point;
use graphics::size::Size;
use graphics::window::Window;

use programs::package::*;
use programs::session::*;

use schemes::arp::*;
use schemes::context::*;
use schemes::debug::*;
use schemes::ethernet::*;
use schemes::file::*;
use schemes::http::*;
use schemes::icmp::*;
use schemes::ip::*;
use schemes::memory::*;
use schemes::random::*;
use schemes::tcp::*;
use schemes::time::*;
use schemes::udp::*;

use syscall::call;
use syscall::handle::*;

mod audio {
    pub mod ac97;
    pub mod intelhda;
    pub mod wav;
}

mod common {
    pub mod context;
    pub mod debug;
    pub mod elf;
    pub mod event;
    pub mod queue;
    pub mod memory;
    pub mod mutex;
    pub mod paging;
    pub mod random;
    pub mod resource;
    pub mod scheduler;
    pub mod string;
    pub mod time;
    pub mod vec;
}

mod drivers {
    pub mod disk;
    pub mod mmio;
    pub mod pci;
    pub mod pciconfig;
    pub mod pio;
    pub mod ps2;
    pub mod rtc;
    pub mod serial;
}

pub mod externs;

mod graphics {
    pub mod bmp;
    pub mod color;
    pub mod display;
    pub mod point;
    pub mod size;
    pub mod window;
}

mod network {
    pub mod arp;
    pub mod common;
    pub mod ethernet;
    pub mod icmp;
    pub mod intel8254x;
    pub mod ipv4;
    pub mod rtl8139;
    pub mod scheme;
    pub mod tcp;
    pub mod udp;
}

mod programs {
    pub mod common;
    pub mod executor;
    pub mod package;
    pub mod session;
}

mod schemes {
    pub mod arp;
    pub mod context;
    pub mod debug;
    pub mod ethernet;
    pub mod file;
    pub mod http;
    pub mod icmp;
    pub mod ip;
    pub mod memory;
    pub mod random;
    pub mod tcp;
    pub mod time;
    pub mod udp;
}

mod syscall {
    pub mod call;
    pub mod common;
    pub mod handle;
}

mod usb {
    pub mod ehci;
    pub mod uhci;
    pub mod xhci;
}

static mut debug_display: *mut Box<Display> = 0 as *mut Box<Display>;
static mut debug_point: Point = Point { x: 0, y: 0 };
static mut debug_draw: bool = false;
static mut debug_redraw: bool = false;
static mut debug_command: *mut String = 0 as *mut String;

static mut clock_realtime: Duration = Duration {
    secs: 0,
    nanos: 0
};

static mut clock_monotonic: Duration = Duration {
    secs: 0,
    nanos: 0
};

static PIT_DURATION: Duration = Duration {
    secs: 0,
    nanos: 2250286
};

static mut session_ptr: *mut Box<Session> = 0 as *mut Box<Session>;

static mut events_ptr: *mut Queue<Event> = 0 as *mut Queue<Event>;

unsafe fn idle_loop() -> ! {
    loop {
        asm!("cli");

        let mut halt = true;

        let contexts = & *contexts_ptr;
        for i in 1..contexts.len() {
            match contexts.get(i) {
                Option::Some(context) => if context.interrupted {
                    halt = false;
                    break;
                },
                Option::None => ()
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

unsafe fn poll_loop() -> ! {
    let session = &mut *session_ptr;

    loop {
        session.on_poll();

        call::sys_yield();
    }
}

unsafe fn event_loop() -> ! {
    let session = &mut *session_ptr;
    let events = &mut *events_ptr;
    let mut cmd = String::new();
    loop {
        loop {
            let reenable = start_no_ints();

            let event_option = events.pop();

            end_no_ints(reenable);

            match event_option {
                Option::Some(event) => {
                    if debug_draw {
                        match event.to_option() {
                            EventOption::Key(key_event) => {
                                if key_event.pressed {
                                    match key_event.scancode {
                                        event::K_F2 => {
                                            ::debug_draw = false;
                                            (*::session_ptr).redraw = cmp::max((*::session_ptr).redraw, event::REDRAW_ALL);
                                        },
                                        event::K_BKSP => if cmd.len() > 0 {
                                            debug::db(8);
                                            cmd.vec.pop();
                                        },
                                        _ => match key_event.character {
                                            '\0' => (),
                                            '\n' => {
                                                let reenable = start_no_ints();
                                                *::debug_command = cmd + '\n';
                                                end_no_ints(reenable);

                                                cmd = String::new();
                                                debug::dl();
                                            }
                                            _ => {
                                                cmd.vec.push(key_event.character);
                                                debug::dc(key_event.character);
                                            }
                                        }
                                    }
                                }
                            },
                            _ => ()
                        }
                    } else {
                        if event.code == 'k' && event.b as u8 == event::K_F1 && event.c > 0 {
                            ::debug_draw = true;
                            ::debug_redraw = true;
                        } else {
                            session.event(event);
                        }
                    }
                },
                Option::None => break
            }
        }

        call::sys_yield();
    }
}

unsafe fn redraw_loop() -> ! {
    let session = &mut *session_ptr;

    loop {
        if debug_draw {
            let display = &*(*debug_display);
            if debug_redraw {
                debug_redraw = false;
                display.flip();
            }
        } else {
            session.redraw();
        }

        call::sys_yield();
    }
}

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

unsafe fn test_disk(disk: Disk) {
    if disk.identify() {
        debug::d(" Disk Found");

        let fs = FileSystem::from_disk(disk);
        if fs.valid() {
            debug::d(" Redox Filesystem");
        } else {
            debug::d(" Unknown Filesystem");
        }
    } else {
        debug::d(" Disk Not Found");
    }
    debug::dl();
}

unsafe fn init(font_data: usize) {
    start_no_ints();

    debug_display = 0 as *mut Box<Display>;
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

    session_ptr = 0 as *mut Box<Session>;

    events_ptr = 0 as *mut Queue<Event>;

    debug_init();

    debug::dd(mem::size_of::<usize>() * 8);
    debug::d(" bits");
    debug::dl();

    Page::init();
    memory::cluster_init();
    //Unmap first page to catch null pointer errors (after reading memory map)
    Page::new(0).unmap();

    ptr::write(display::FONTS, font_data);

    debug_display = memory::alloc_type();
    ptr::write(debug_display, box Display::root());
    (*debug_display).set(Color::new(0, 0, 0));
    debug_draw = true;
    debug_command = memory::alloc_type();
    ptr::write(debug_command, String::new());

    clock_realtime = RTC::new().time();

    contexts_ptr = memory::alloc_type();
    ptr::write(contexts_ptr, Vec::new());
    (*contexts_ptr).push(Context::root());

    session_ptr = memory::alloc_type();
    ptr::write(session_ptr, box Session::new());

    events_ptr = memory::alloc_type();
    ptr::write(events_ptr, Queue::new());

    let session = &mut *session_ptr;

    session.items.push(PS2::new());
    session.items.push(box Serial::new(0x3F8, 0x4));

    pci_init(session);

    debug::d("Primary Master:");
    test_disk(Disk::primary_master());

    debug::d("Primary Slave:");
    test_disk(Disk::primary_slave());

    debug::d("Secondary Master:");
    test_disk(Disk::secondary_master());

    debug::d("Secondary Slave:");
    test_disk(Disk::secondary_slave());

    session.items.push(box ContextScheme);
    session.items.push(box DebugScheme);
    session.items.push(box FileScheme {
        fs: FileSystem::from_disk(Disk::primary_master())
    });
    session.items.push(box HTTPScheme);
    session.items.push(box MemoryScheme);
    session.items.push(box RandomScheme);
    session.items.push(box TimeScheme);

    session.items.push(box EthernetScheme);
    session.items.push(box ARPScheme);
    session.items.push(box IPScheme {
        arp: Vec::new()
    });
    session.items.push(box ICMPScheme);
    session.items.push(box TCPScheme);
    session.items.push(box UDPScheme);

    Context::spawn(box move || {
        poll_loop();
    });
    Context::spawn(box move || {
        event_loop();
    });
    Context::spawn(box move || {
        redraw_loop();
    });
    Context::spawn(box move || {
        ARPScheme::reply_loop();
    });
    Context::spawn(box move || {
        ICMPScheme::reply_loop();
    });

    //Start interrupts
    end_no_ints(true);

    {
        let mut resource = URL::from_str("file:///ui/cursor.bmp").open();

        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);
        session.cursor = BMP::from_data(&vec);
    }

    {
        let mut resource = URL::from_str("file:///ui/background.bmp").open();

        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);
        session.background = BMP::from_data(&vec)
    }

    debug_draw = false;

    session.redraw = cmp::max(session.redraw, event::REDRAW_ALL);

    {
        let mut resource = URL::from_str("file:///apps/").open();

        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);

        for folder in String::from_utf8(&vec).split("\n".to_string()) {
            if folder.ends_with("/".to_string()) {
                session.packages.push(Package::from_url(&URL::from_string(&("file:///apps/".to_string() + folder))));
            }
        }
    }
}

fn dr(reg: &str, value: u32) {
    debug::d(reg);
    debug::d(": ");
    debug::dh(value as usize);
    debug::dl();
}

#[no_mangle]
//Take regs for kernel calls and exceptions
pub unsafe fn kernel(interrupt: u32, edi: u32, esi: u32, ebp: u32, esp: u32, ebx: u32, edx: u32, ecx: u32, mut eax: u32, eip: u32, eflags: u32, error: u32) -> u32 {
    macro_rules! exception {
        ($name:expr) => ({
            debug::d($name);
            debug::dl();

            dr("CONTEXT", context_i as u32);
            dr("EFLAGS", eflags);
            dr("EIP", eip);
            dr("EAX", eax);
            dr("ECX", ecx);
            dr("EDX", edx);
            dr("EBX", ebx);
            dr("ESP", esp);
            dr("EBP", ebp);
            dr("ESI", esi);
            dr("EDI", edi);
            dr("INT", interrupt);

            let cr0;
            asm!("mov eax, cr0" : "={eax}"(cr0) : : : "intel", "volatile");
            dr("CR0", cr0);

            let cr2;
            asm!("mov eax, cr2" : "={eax}"(cr2) : : : "intel", "volatile");
            dr("CR2", cr2);

            let cr3;
            asm!("mov eax, cr3" : "={eax}"(cr3) : : : "intel", "volatile");
            dr("CR3", cr3);

            let cr4;
            asm!("mov eax, cr4" : "={eax}"(cr4) : : : "intel", "volatile");
            dr("CR4", cr4);

            call::sys_exit(-1);
            loop {
                asm!("sti");
                asm!("hlt");
            }
        })
    };

    macro_rules! exception_error {
        ($name:expr) => ({
            debug::d($name);
            debug::dl();

            dr("CONTEXT", context_i as u32);
            dr("EFLAGS", error);
            dr("EIP", eflags);
            dr("ERROR", eip);
            dr("EAX", eax);
            dr("ECX", ecx);
            dr("EDX", edx);
            dr("EBX", ebx);
            dr("ESP", esp);
            dr("EBP", ebp);
            dr("ESI", esi);
            dr("EDI", edi);
            dr("INT", interrupt);

            let cr0;
            asm!("mov eax, cr0" : "={eax}"(cr0) : : : "intel", "volatile");
            dr("CR0", cr0);

            let cr2;
            asm!("mov eax, cr2" : "={eax}"(cr2) : : : "intel", "volatile");
            dr("CR2", cr2);

            let cr3;
            asm!("mov eax, cr3" : "={eax}"(cr3) : : : "intel", "volatile");
            dr("CR3", cr3);

            let cr4;
            asm!("mov eax, cr4" : "={eax}"(cr4) : : : "intel", "volatile");
            dr("CR4", cr4);

            call::sys_exit(-1);
            loop {
                asm!("sti");
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
            let reenable = start_no_ints();
            clock_realtime = clock_realtime + PIT_DURATION;
            clock_monotonic = clock_monotonic + PIT_DURATION;
            end_no_ints(reenable);

            context_switch(true);
        }
        0x21 => (*session_ptr).on_irq(0x1), //keyboard
        0x23 => (*session_ptr).on_irq(0x3), // serial 2 and 4
        0x24 => (*session_ptr).on_irq(0x4), // serial 1 and 3
        0x28 => (*session_ptr).on_irq(0x8), //RTC
        0x29 => (*session_ptr).on_irq(0x9), //pci
        0x2A => (*session_ptr).on_irq(0xA), //pci
        0x2B => (*session_ptr).on_irq(0xB), //pci
        0x2C => (*session_ptr).on_irq(0xC), //mouse
        0x2E => (*session_ptr).on_irq(0xE), //disk
        0x2F => (*session_ptr).on_irq(0xF), //disk
        0x80 => eax = syscall_handle(eax, ebx, ecx, edx),
        0xFF => {
            init(eax as usize);
            context_enabled = true;
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
        _ => {
            debug::d("Interrupt: ");
            debug::dh(interrupt as usize);
            debug::dl();
        }
    }

    eax
}
