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
#![feature(rc_unique)]
#![feature(unboxed_closures)]
#![feature(unsafe_no_drop_flag)]
#![no_std]

extern crate alloc;

#[macro_use]
extern crate mopa;

use core::fmt;
use core::mem::size_of;
use core::ptr;

use alloc::boxed::*;
use alloc::rc::*;

use common::debug::*;
use common::pio::*;
use common::memory::*;
use common::resource::*;
use common::string::*;
use common::url::*;
use common::vec::*;

use drivers::keyboard::*;
use drivers::mouse::*;
use drivers::pci::*;
use drivers::ps2::*;
use drivers::serial::*;

use graphics::bmp::*;

use programs::executor::*;
use programs::filemanager::*;
use programs::session::*;
use programs::syscall;

use schemes::file::*;
use schemes::http::*;
use schemes::memory::*;
use schemes::pci::*;
use schemes::random::*;

mod common {
    pub mod debug;
    pub mod elf;
    pub mod memory;
    pub mod pci;
    pub mod pio;
    pub mod random;
    pub mod resource;
    pub mod string;
    pub mod vec;
    pub mod url;
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
    pub mod editor;
    pub mod executor;
    pub mod filemanager;
    pub mod session;
    pub mod syscall;
    pub mod viewer;
}

mod schemes {
    pub mod file;
    pub mod http;
    pub mod ide;
    pub mod memory;
    pub mod pci;
    pub mod random;
}

mod usb {
    pub mod xhci;
}

static mut session_ptr: *mut Session = 0 as *mut Session;

unsafe fn init(){
    serial_init();

    dd(size_of::<usize>() * 8);
    d(" bits");
    dl();

    page_init();
    cluster_init();

    session_ptr = alloc(size_of::<Session>()) as *mut Session;
    *session_ptr = Session::new();
    let session = &mut *session_ptr;

    session.items.insert(0, Rc::new(FileManager::new()));

    keyboard_init();
    mouse_init();

    session.modules.push(Rc::new(PS2));
    session.modules.push(Rc::new(Serial::new(0x3F8, 0x4)));

    pci_init(session);

    session.modules.push(Rc::new(FileScheme));
    session.modules.push(Rc::new(HTTPScheme));
    session.modules.push(Rc::new(MemoryScheme));
    session.modules.push(Rc::new(PCIScheme));
    session.modules.push(Rc::new(RandomScheme));

    syscall::open_async(&URL::from_string("file:///background.bmp".to_string()), box |mut resource: Box<Resource>|{
        d("\nfile://background.bmp return\n");
        let mut vec: Vec<u8> = Vec::new();
        d("Read to end start\n");
        match resource.read_to_end(&mut vec) {
            Option::Some(0) => d("No background data\n"),
            Option::Some(len) => {
                d("Background load ");
                dh(vec.as_ptr() as usize);
                d(" ");
                dd(vec.len());
                dl();

                (*session_ptr).display.background = BMP::from_data(vec.as_ptr() as usize);

                d("Background is ");
                dd((*session_ptr).display.background.size.width);
                d(" x ");
                dd((*session_ptr).display.background.size.height);
                dl();
            },
            Option::None => d("Background load error\n")
        }
        d("Read to end end\n");
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

        asm!("cli");
        asm!("hlt");
        loop{}
    };

    match interrupt {
        0x20 => (), //timer
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
                0x1 => {
                    d("Open: ");
                    let url: &URL = &*(ebx as *const URL);
                    url.d();

                    let session = &mut *session_ptr;
                    ptr::write(ecx as *mut Box<Resource>, session.open(url));
                },
                0x2 => {
                    d("Open Async: ");
                    let url: &URL = &*(ebx as *const URL);
                    let callback: Box<FnBox(Box<Resource>)> = ptr::read(ecx as *const Box<FnBox(Box<Resource>)>);
                    unalloc(ecx as usize);
                    url.d();

                    let session = &mut *session_ptr;
                    session.open_async(url, callback);
                },
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
        0xFF => { // main loop
            init();

            loop {
                (*session_ptr).on_poll();
                (*session_ptr).redraw();
                asm!("sti");
                asm!("hlt");
                asm!("cli"); // TODO: Allow preempting
            }
        },
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

    if interrupt >= 0x20 && interrupt < 0x30 {
        if interrupt >= 0x28 {
            outb(0xA0, 0x20);
        }

        outb(0x20, 0x20);
    }
}

#[lang = "panic_fmt"]
pub extern fn panic_fmt(fmt: fmt::Arguments, file: &'static str, line: u32) -> ! {
    d("PANIC: ");
    d(file);
    d(": ");
    dh(line as usize);
    dl();
    unsafe{
        asm!("cli");
        asm!("hlt");
    }
    loop{}
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
