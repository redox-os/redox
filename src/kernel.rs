#![feature(alloc)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(core_simd)]
#![feature(core_slice_ext)]
#![feature(core_str_ext)]
#![feature(fnbox)]
#![feature(fundamental)]
#![feature(lang_items)]
#![feature(no_std)]
#![feature(unboxed_closures)]
#![feature(unsafe_no_drop_flag)]
#![no_std]

extern crate alloc;

#[macro_use]
extern crate mopa;

use core::cmp::max;
use core::mem::size_of;
use core::mem::swap;
use core::ptr;

use common::context::*;
use common::pio::*;
use common::memory::*;
use common::paging::*;

use drivers::disk::*;
use drivers::keyboard::keyboard_init;
use drivers::mouse::mouse_init;
use drivers::pci::*;
use drivers::ps2::*;
use drivers::serial::*;

use filesystems::unfs::*;

use graphics::bmp::*;
use graphics::color::*;

use programs::common::*;
use programs::session::*;

use schemes::debug::*;
use schemes::file::*;
use schemes::http::*;
use schemes::memory::*;
use schemes::pci::*;
use schemes::random::*;

mod common {
    pub mod context;
    pub mod debug;
    pub mod elf;
    pub mod event;
    pub mod memory;
    pub mod paging;
    pub mod pci;
    pub mod pio;
    pub mod random;
    pub mod resource;
    pub mod string;
    pub mod vec;
}

mod drivers {
    pub mod disk;
    pub mod keyboard;
    pub mod mouse;
    pub mod pci;
    pub mod ps2;
    pub mod serial;
}

mod filesystems {
    pub mod unfs;
}

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
    pub mod tcp;
    pub mod udp;
}

mod programs {
    pub mod common;
    pub mod editor;
    pub mod executor;
    pub mod filemanager;
    pub mod session;
    pub mod viewer;
}

mod schemes {
    pub mod debug;
    pub mod file;
    pub mod http;
    pub mod ide;
    pub mod memory;
    pub mod pci;
    pub mod random;
}

mod usb {
    pub mod ehci;
    pub mod xhci;
}

static mut debug_display: *mut Box<Display> = 0 as *mut Box<Display>;
static mut debug_point: Point = Point{ x: 0, y: 0 };
static mut debug_draw: bool = false;
static mut debug_redraw: bool = false;

static mut contexts_ptr: *mut Vec<Context> = 0 as *mut Vec<Context>;
static mut context_i: usize = 0;

static mut session_ptr: *mut Box<Session> = 0 as *mut Box<Session>;

static mut events_ptr: *mut Vec<Event> = 0 as *mut Vec<Event>;

pub unsafe extern "C" fn event_loop() -> ! {
    let session = &mut *session_ptr;
    loop {
        asm!("cli");
        session.on_poll();

        let mut events_copy: Vec<Event> = Vec::new();
        swap(&mut events_copy, &mut *events_ptr);
        asm!("sti");

        session.handle_events(&mut events_copy);

        sched_yield();
    }
}

pub unsafe extern "C" fn redraw_loop() -> ! {
    let session = &mut *session_ptr;

    {
        let mut resource = URL::from_string("file:///background.bmp".to_string()).open();

        let mut vec: Vec<u8> = Vec::new();
        match resource.read_to_end(&mut vec) {
            Option::Some(_) => session.background = BMP::from_data(vec.as_ptr() as usize),
            Option::None => d("Background load error\n")
        }
    }

    loop {
        asm!("cli");
        if debug_draw {
            if debug_redraw {
                debug_redraw = false;
                (*debug_display).flip();
            }
        }else{
            session.redraw();
        }
        asm!("sti");

        sched_yield();
    }
}

unsafe fn context_switch(){
    if contexts_ptr as usize > 0 {
        let contexts = &*contexts_ptr;
        let current_i = context_i;
        context_i += 1;
        if context_i >= contexts.len(){
            context_i -= contexts.len();
        }
        if context_i != current_i {
            match contexts.get(current_i){
                Option::Some(current) => match contexts.get(context_i) {
                    Option::Some(next) => {
                        current.swap(next);
                    },
                    Option::None => ()
                },
                Option::None => ()
            }
        }
    }
}

unsafe fn test_disk(disk: Disk){
    if disk.identify() {
        d(" Disk Found");

        let unfs = UnFS::from_disk(disk);
        if unfs.valid() {
            d(" UnFS Filesystem");
        }else{
            d(" Unknown Filesystem");
        }
    }else{
        d(" Disk Not Found");
    }
    dl();
}

unsafe fn init(font_data: usize, cursor_data: usize){
    debug_display = 0 as *mut Box<Display>;
    debug_point = Point{ x: 0, y: 0 };
    debug_draw = false;
    debug_redraw = false;

    contexts_ptr = 0 as *mut Vec<Context>;
    context_i = 0;

    session_ptr = 0 as *mut Box<Session>;

    events_ptr = 0 as *mut Vec<Event>;

    debug_init();

    dd(size_of::<usize>() * 8);
    d(" bits");
    dl();

    page_init();
    cluster_init();

    *FONTS = font_data;

    debug_display = alloc(size_of::<Box<Display>>()) as *mut Box<Display>;
    ptr::write(debug_display, box Display::root());
    (*debug_display).set(Color::new(0, 0, 0));
    debug_draw = true;

    contexts_ptr = alloc(size_of::<Vec<Context>>()) as *mut Vec<Context>;
    ptr::write(contexts_ptr, Vec::new());

    let contexts = &mut *contexts_ptr;
    contexts.push(Context::root());

    session_ptr = alloc(size_of::<Box<Session>>()) as *mut Box<Session>;
    ptr::write(session_ptr, box Session::new());

    events_ptr = alloc(size_of::<Vec<Event>>()) as *mut Vec<Event>;
    ptr::write(events_ptr, Vec::new());

    let session = &mut *session_ptr;
    session.cursor = BMP::from_data(cursor_data);

    keyboard_init();
    mouse_init();

    session.items.push(box PS2);
    session.items.push(box Serial::new(0x3F8, 0x4));

    pci_init(session);

    d("Primary Master:");
    test_disk(Disk::primary_master());

    d("Primary Slave:");
    test_disk(Disk::primary_slave());

    d("Secondary Master:");
    test_disk(Disk::secondary_master());

    d("Secondary Slave:");
    test_disk(Disk::secondary_slave());

    session.items.push(box DebugScheme);
    session.items.push(box FileScheme{
        unfs: UnFS::from_disk(Disk::primary_master())
    });
    session.items.push(box HTTPScheme);
    session.items.push(box MemoryScheme);
    session.items.push(box PCIScheme);
    session.items.push(box RandomScheme);

    (*contexts_ptr).push(Context::new(event_loop));
    (*contexts_ptr).push(Context::new(redraw_loop));
}

fn dr(reg: &str, value: u32){
    d(reg);
    d(": ");
    dh(value as usize);
    dl();
}

#[no_mangle]
//Take regs for kernel calls and exceptions
pub unsafe fn kernel(interrupt: u32, edi: u32, esi: u32, ebp: u32, esp: u32, ebx: u32, edx: u32, ecx: u32, eax: u32, eip: u32, eflags: u32) {
    let exception = |name: &str|{
        d(name);
        dl();

        dr("INT", interrupt);
        dr("EIP", eip);
        dr("EFLAGS", eflags);
        dr("EAX", eax);
        dr("EBX", ebx);
        dr("ECX", ecx);
        dr("EDX", edx);
        dr("EDI", edi);
        dr("ESI", esi);
        dr("EBP", ebp);
        dr("ESP", esp);

        loop {
            asm!("cli");
            asm!("hlt");
        }
    };

    if interrupt >= 0x20 && interrupt < 0x30 {
        if interrupt >= 0x28 {
            outb(0xA0, 0x20);
        }

        outb(0x20, 0x20);
    }

    match interrupt {
        0x20 => context_switch(), // Context switch timer
        0x21 => (*session_ptr).on_irq(0x1), //keyboard
        0x23 => (*session_ptr).on_irq(0x3), // serial 2 and 4
        0x24 => (*session_ptr).on_irq(0x4), // serial 1 and 3
        0x29 => (*session_ptr).on_irq(0x9), //pci
        0x2A => (*session_ptr).on_irq(0xA), //pci
        0x2B => (*session_ptr).on_irq(0xB), //pci
        0x2C => (*session_ptr).on_irq(0xC), //mouse
        0x2E => (*session_ptr).on_irq(0xE), //disk
        0x2F => (*session_ptr).on_irq(0xF), //disk
        0x80 => { // kernel calls
            match eax {
                0x0 => { //Debug
                    if debug_display as usize > 0 {
                        if ebx == 10 {
                            debug_point.x = 0;
                            debug_point.y += 16;
                            debug_redraw = true;
                        }else{
                            (*debug_display).char(debug_point, (ebx as u8) as char, Color::new(255, 255, 255));
                            debug_point.x += 8;
                        }
                        if debug_point.x >= (*debug_display).width as isize {
                            debug_point.x = 0;
                            debug_point.y += 16;
                        }
                        while debug_point.y + 16 > (*debug_display).height as isize {
                            (*debug_display).scroll(16);
                            debug_point.y -= 16;
                        }
                        if debug_draw && debug_redraw {
                            debug_redraw = false;
                            (*debug_display).flip();
                        }
                    }else{
                        outb(0x3F8, ebx as u8);
                    }
                }
                0x1 => {
                    d("Open: ");
                    let url: &URL = &*(ebx as *const URL);
                    url.d();

                    let session = &mut *session_ptr;
                    ptr::write(ecx as *mut Box<Resource>, session.open(url));
                },
                0x2 => {
                    let event = *(ebx as *const Event);

                    if event.code == 'k' && event.b == 0x3B && event.c > 0 {
                        debug_draw = true;
                        debug_redraw = true;
                    }
                    if event.code == 'k' && event.b == 0x3C && event.c > 0 {
                        debug_draw = false;
                        (*session_ptr).redraw = max((*session_ptr).redraw, REDRAW_ALL);
                    }

                    (*events_ptr).push(event);
                },
                0x3 => context_switch(),
                _ => {
                    d("System Call");
                    d(" EAX:");
                    dh(eax as usize);
                    d(" EBX:");
                    dh(ebx as usize);
                    d(" ECX:");
                    dh(ecx as usize);
                    d(" EDX:");
                    dh(edx as usize);
                    dl();
                }
            }
        },
        0xFF => init(eax as usize, ebx as usize),
        0x0 => exception("Divide by zero exception"),
        0x1 => exception("Debug exception"),
        0x2 => exception("Non-maskable interrupt"),
        0x3 => exception("Breakpoint exception"),
        0x4 => exception("Overflow exception"),
        0x5 => exception("Bound range exceeded exception"),
        0x6 => exception("Invalid opcode exception"),
        0x7 => exception("Device not available exception"),
        0x8 => exception("Double fault"),
        0xA => exception("Invalid TSS exception"),
        0xB => exception("Segment not present exception"),
        0xC => exception("Stack-segment fault"),
        0xD => exception("General protection fault"),
        0xE => exception("Page fault"),
        0x10 => exception("x87 floating-point exception"),
        0x11 => exception("Alignment check exception"),
        0x12 => exception("Machine check exception"),
        0x13 => exception("SIMD floating-point exception"),
        0x14 => exception("Virtualization exception"),
        0x1E => exception("Security exception"),
        _ => {
            d("Interrupt: ");
            dh(interrupt as usize);
            dl();
        }
    }
}

#[no_mangle]
pub extern "C" fn memcmp(a: *mut u8, b: *const u8, len: isize) -> isize {
    unsafe {
        let mut i = 0;
        while i < len {
            let c_a = *a.offset(i);
            let c_b = *b.offset(i);
            if c_a != c_b{
                return c_a as isize - c_b as isize;
            }
            i += 1;
        }
        return 0;
    }
}

#[no_mangle]
pub extern "C" fn memmove(dst: *mut u8, src: *const u8, len: isize){
    unsafe {
        if src < dst {
            let mut i = len;
            while i > 0 {
                i -= 1;
                *dst.offset(i) = *src.offset(i);
            }
        }else{
            let mut i = 0;
            while i < len {
                *dst.offset(i) = *src.offset(i);
                i += 1;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn memcpy(dst: *mut u8, src: *const u8, len: isize){
    unsafe {
        let mut i = 0;
        while i < len {
            *dst.offset(i) = *src.offset(i);
            i += 1;
        }
    }
}

#[no_mangle]
pub extern "C" fn memset(src: *mut u8, c: i32, len: isize) {
    unsafe {
        let mut i = 0;
        while i < len {
            *src.offset(i) = c as u8;
            i += 1;
        }
    }
}
