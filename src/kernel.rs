#![feature(asm)]
#![feature(box_syntax)]
#![feature(coerce_unsized)]
#![feature(core)]
#![feature(core_prelude)]
#![feature(core_simd)]
#![feature(core_slice_ext)]
#![feature(core_str_ext)]
#![feature(fundamental)]
#![feature(lang_items)]
#![feature(no_std)]
#![feature(raw)]
#![feature(unique)]
#![feature(unsize)]
#![no_std]

extern crate core;

use core::mem::size_of;

use common::debug::*;
use common::pio::*;
use common::memory::*;
use common::string::*;
use common::url::*;

use drivers::keyboard::*;
use drivers::mouse::*;
use drivers::pci::*;
use drivers::ps2::*;
use drivers::serial::*;

use graphics::bmp::*;

use programs::filemanager::*;
use programs::session::*;

use schemes::file::*;
use schemes::http::*;
use schemes::memory::*;
use schemes::pci::*;
use schemes::random::*;

mod alloc {
    pub mod boxed;
}

mod common {
    pub mod debug;
    pub mod elf;
    pub mod memory;
    pub mod pci;
    pub mod pio;
    pub mod random;
    pub mod safeptr;
    pub mod string;
    pub mod vector;
    pub mod url;
}

mod drivers {
    pub mod disk;
    pub mod ide;
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
    pub mod http;
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
    pub mod viewer;
}

mod schemes {
    pub mod file;
    pub mod http;
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

    session.items.insert(0, box FileManager::new("".to_string()));

    keyboard_init();
    mouse_init();

    session.modules.push(box PS2);
    session.modules.push(box Serial::new(0x3F8, 0x4));

    pci_init(session);

    session.modules.push(box FileScheme);
    session.modules.push(box HTTPScheme);
    session.modules.push(box MemoryScheme);
    session.modules.push(box PCIScheme);
    session.modules.push(box RandomScheme);

    session.on_url_async(&URL::from_string("file:///background.bmp".to_string()), box |response: String|{
        dl();
        let background_data = response.to_num();
        dh(background_data);
        d(" ");
        dd(alloc_size(background_data)/1024);
        d(" KB");
        dl();
        d("Response File\n");
        if background_data > 0 {
            (*session_ptr).display.background = BMP::from_data(background_data);
            unalloc(background_data);
        }
    });
}

#[no_mangle]
pub unsafe fn kernel(interrupt: u32) {
    match interrupt {
        0x20 => (), //timer
        0x21 => (*session_ptr).on_irq(0x1), //keyboard
        0x23 => (*session_ptr).on_irq(0x3), // serial 2 and 4
        0x24 => (*session_ptr).on_irq(0x4), // serial 1 and 3
        0x2B => (*session_ptr).on_irq(0xB), //pci
        0x2C => (*session_ptr).on_irq(0xC), //mouse
        0x2E => (*session_ptr).on_irq(0xE), //disk
        0x2F => (*session_ptr).on_irq(0xF), //disk
        0xFF => { // main loop
            init();

            loop {
                (*session_ptr).on_poll();
                (*session_ptr).redraw();
                asm!("sti");
                asm!("hlt");
                asm!("cli"); // TODO: Allow preempting
            }
        }
        _ => {
            d("I: ");
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
pub extern "C" fn memset(src: *mut u8, c: i32, len: isize) {
    unsafe {
        let mut i = 0;
        while i < len {
            *src.offset(i) = c as u8;
            i += 1;
        }
    }
}
