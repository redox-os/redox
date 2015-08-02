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

use drivers::keyboard::*;
use drivers::mouse::*;
use drivers::pci::*;

use programs::filemanager::*;
use programs::session::*;

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
    pub mod keyboard;
    pub mod mouse;
    pub mod pci;
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

mod usb {
    pub mod xhci;
}

static mut session: *mut Session = 0 as *mut Session;

unsafe fn init(){
    serial_init();

    dd(size_of::<usize>() * 8);
    d(" bits");
    dl();

    page_init();
    cluster_init();

    session = alloc(size_of::<Session>()) as *mut Session;
    *session = Session::new();
    (*session).items.insert(0, box FileManager::new("".to_string()));

    keyboard_init();
    mouse_init();

    pci_init(&mut *session);
}

pub unsafe fn input_handle(){
    loop {
        let status = inb(0x64);
        if status & 0x21 == 1 {
            let key_event = keyboard_interrupt();
            if key_event.scancode > 0 {
                (*session).on_key(key_event);
            }
        }else if status & 0x21 == 0x21 {
            let mouse_event = mouse_interrupt();

            if mouse_event.valid {
                (*session).on_mouse(mouse_event);
            }
        }else{
            break;
        }
    }
}

pub unsafe fn pci_handle(irq: u8){
    if cfg!(debug_pci){
        d("PCI Handle ");
        dh(irq as usize);
        dl();
    }

    (*session).on_irq(irq);
}


#[no_mangle]
pub unsafe fn kernel(interrupt: u32) {
    match interrupt {
        0x20 => (), //timer
        0x21 => input_handle(), //keyboard
        0x2B => pci_handle(0xB),
        0x2C => input_handle(), //mouse
        0x2E => (), //disk
        0xFF => { // main loop
            init();

            loop {
                (*session).redraw();
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
