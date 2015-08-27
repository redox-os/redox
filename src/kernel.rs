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

use common::context::*;
use common::memory::*;
use common::paging::*;
use common::pio::*;

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

use syscall::common::*;
use syscall::handle::*;

mod common {
    pub mod context;
    pub mod debug;
    pub mod elf;
    pub mod event;
    pub mod queue;
    pub mod memory;
    pub mod mutex;
    pub mod paging;
    pub mod pci;
    pub mod pio;
    pub mod random;
    pub mod resource;
    pub mod scheduler;
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

mod syscall {
    pub mod call;
    pub mod common;
    pub mod handle;
}

mod usb {
    pub mod ehci;
    pub mod xhci;
}

static mut debug_display: *mut Box<Display> = 0 as *mut Box<Display>;
static mut debug_point: Point = Point{ x: 0, y: 0 };
static mut debug_draw: bool = false;
static mut debug_redraw: bool = false;

static mut session_ptr: *mut Box<Session> = 0 as *mut Box<Session>;

static mut events_ptr: *mut Box<Queue<Event>> = 0 as *mut Box<Queue<Event>>;

unsafe fn poll_loop() -> ! {
    let session = &mut *session_ptr;
    loop {
        let reenable = start_no_ints();

        session.on_poll();

        end_no_ints(reenable);

        sys_yield();
    }
}

unsafe fn event_loop() -> ! {
    let session = &mut *session_ptr;
    let events = &mut *events_ptr;
    loop {
        let reenable = start_no_ints();

        loop{
            match events.pop() {
                Option::Some(event) => session.event(event),
                Option::None => break
            }
        }

        end_no_ints(reenable);

        sys_yield();
    }
}

unsafe fn redraw_loop() -> ! {
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
        if debug_draw {
            let display = &*(*debug_display);
            if debug_redraw {
                debug_redraw = false;
                display.flip();
            }
        }else{
            session.redraw();
        }

        sys_yield();
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

    contexts_ptr = 0 as *mut Box<Vec<Context>>;
    context_i = 0;

    session_ptr = 0 as *mut Box<Session>;

    events_ptr = 0 as *mut Box<Queue<Event>>;

    debug_init();

    dd(size_of::<usize>() * 8);
    d(" bits");
    dl();

    page_init();
    cluster_init();

    *FONTS = font_data;

    debug_display = alloc_type();
    ptr::write(debug_display, box Display::root());
    (*debug_display).set(Color::new(0, 0, 0));
    debug_draw = true;

    contexts_ptr = alloc_type();
    ptr::write(contexts_ptr, box Vec::new());

    session_ptr = alloc_type();
    ptr::write(session_ptr, box Session::new());

    events_ptr = alloc_type();
    ptr::write(events_ptr, box Queue::new());

    let session = &mut *session_ptr;
    session.cursor = BMP::from_data(cursor_data);

    keyboard_init();
    mouse_init();

    session.items.push(Arc::new(PS2));
    session.items.push(Arc::new(Serial::new(0x3F8, 0x4)));

    pci_init(session);

    d("Primary Master:");
    test_disk(Disk::primary_master());

    d("Primary Slave:");
    test_disk(Disk::primary_slave());

    d("Secondary Master:");
    test_disk(Disk::secondary_master());

    d("Secondary Slave:");
    test_disk(Disk::secondary_slave());

    session.items.push(Arc::new(DebugScheme));
    session.items.push(Arc::new(FileScheme{
        unfs: UnFS::from_disk(Disk::primary_master())
    }));
    session.items.push(Arc::new(HTTPScheme));
    session.items.push(Arc::new(MemoryScheme));
    session.items.push(Arc::new(PCIScheme));
    session.items.push(Arc::new(RandomScheme));

    (*contexts_ptr).push(Context::root());
    Context::spawn(box move ||{
        poll_loop();
    });
    Context::spawn(box move ||{
        event_loop();
    });
    Context::spawn(box move ||{
        redraw_loop();
    });
}

fn dr(reg: &str, value: u32){
    d(reg);
    d(": ");
    dh(value as usize);
    dl();
}

#[no_mangle]
//Take regs for kernel calls and exceptions
pub unsafe extern "cdecl" fn kernel(interrupt: u32, edi: u32, esi: u32, ebp: u32, esp: u32, ebx: u32, edx: u32, ecx: u32, eax: u32, eip: u32, eflags: u32) {
    macro_rules! exception {
        ($name:expr) => ({
            d($name);
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
        })
    };

    if interrupt >= 0x20 && interrupt < 0x30 {
        if interrupt >= 0x28 {
            outb(0xA0, 0x20);
        }

        outb(0x20, 0x20);
    }

    match interrupt {
        0x20 => syscall_handle(SYS_YIELD, 0, 0, 0), // Context switch timer
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
        0x80 => syscall_handle(eax, ebx, ecx, edx),
        0xFF => init(eax as usize, ebx as usize),
        0x0 => exception!("Divide by zero exception"),
        0x1 => exception!("Debug exception"),
        0x2 => exception!("Non-maskable interrupt"),
        0x3 => exception!("Breakpoint exception"),
        0x4 => exception!("Overflow exception"),
        0x5 => exception!("Bound range exceeded exception"),
        0x6 => exception!("Invalid opcode exception"),
        0x7 => exception!("Device not available exception"),
        0x8 => exception!("Double fault"),
        0xA => exception!("Invalid TSS exception"),
        0xB => exception!("Segment not present exception"),
        0xC => exception!("Stack-segment fault"),
        0xD => exception!("General protection fault"),
        0xE => exception!("Page fault"),
        0x10 => exception!("x87 floating-point exception"),
        0x11 => exception!("Alignment check exception"),
        0x12 => exception!("Machine check exception"),
        0x13 => exception!("SIMD floating-point exception"),
        0x14 => exception!("Virtualization exception"),
        0x1E => exception!("Security exception"),
        _ => {
            d("Interrupt: ");
            dh(interrupt as usize);
            dl();
        }
    }
}

/* Externs { */
#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern fn __rust_allocate(size: usize, align: usize) -> *mut u8{
    return alloc(size) as *mut u8;
}

#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern fn __rust_deallocate(ptr: *mut u8, old_size: usize, align: usize){
    return unalloc(ptr as usize);
}

#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern fn __rust_reallocate(ptr: *mut u8, old_size: usize, size: usize, align: usize) -> *mut u8{
    return realloc(ptr as usize, size) as *mut u8;
}

#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern fn __rust_reallocate_inplace(ptr: *mut u8, old_size: usize, size: usize, align: usize) -> usize{
    return realloc_inplace(ptr as usize, size);
}

#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern fn __rust_usable_size(size: usize, align: usize) -> usize{
    return ((size + CLUSTER_SIZE - 1)/CLUSTER_SIZE) * CLUSTER_SIZE;
}

#[no_mangle]
pub unsafe extern fn memcmp(a: *mut u8, b: *const u8, len: usize) -> isize {
    for i in 0..len {
        let c_a = ptr::read(a.offset(i as isize));
        let c_b = ptr::read(b.offset(i as isize));
        if c_a != c_b{
            return c_a as isize - c_b as isize;
        }
    }
    return 0;
}

#[no_mangle]
pub unsafe extern fn memmove(dst: *mut u8, src: *const u8, len: usize){
    if src < dst {
        asm!("std
            rep movsb"
            :
            : "{edi}"(dst.offset(len as isize - 1)), "{esi}"(src.offset(len as isize - 1)), "{ecx}"(len)
            : "cc", "memory"
            : "intel", "volatile");
    }else{
        asm!("cld
            rep movsb"
            :
            : "{edi}"(dst), "{esi}"(src), "{ecx}"(len)
            : "cc", "memory"
            : "intel", "volatile");
    }
}

#[no_mangle]
pub unsafe extern fn memcpy(dst: *mut u8, src: *const u8, len: usize){
    asm!("cld
        rep movsb"
        :
        : "{edi}"(dst), "{esi}"(src), "{ecx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}

#[no_mangle]
pub unsafe extern fn memset(dst: *mut u8, c: i32, len: usize) {
    asm!("cld
        rep stosb"
        :
        : "{eax}"(c), "{edi}"(dst), "{ecx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}
/* } Externs */
