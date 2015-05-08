#![feature(asm)]
#![feature(box_syntax)]
#![feature(core)]
#![feature(lang_items)]
#![feature(no_std)]
#![no_std]

extern crate core;

use core::mem::size_of;

use common::debug::*;
use common::pio::*;
use common::memory::*;

use drivers::keyboard::*;
use drivers::mouse::*;
use drivers::pci::*;

use programs::filemanager::*;
use programs::session::*;

mod common {
    pub mod debug;
    pub mod elf;
    pub mod memory;
    pub mod pio;
    pub mod string;
    pub mod vector;
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
    pub mod intel8254x;
    pub mod network;
    pub mod rtl8139;
}

mod programs {
    pub mod editor;
    pub mod executor;
    pub mod filemanager;
    pub mod session;
    pub mod viewer;
}

unsafe fn initialize(){
    serial_init();

    dd(size_of::<usize>() * 8);
    d(" bits");
    dl();

    d("Paging\n");
    page_init();

    d("Clusters\n");
    cluster_init();

    d("Mouse\n");
    mouse_init();

    d("Keyboard\n");
    keyboard_init();

    d("PCI\n");
    pci_init();
}

pub unsafe fn timestamp() -> usize {
    let low: u32;
    asm!("rdtsc"
        : "={eax}"(low) : : "{edx}" : "intel");

    return low as usize;
}

const INTERRUPT: *mut u8 = 0x200000 as *mut u8;

#[no_mangle]
pub unsafe fn kernel() {
    if *INTERRUPT == 255 {
        initialize();

        let mut session = Session::new();
        session.add_item(box FileManager::new());

        loop{
            let interrupt = *INTERRUPT;
            *INTERRUPT = 255;

            match interrupt {
                0x20 => (), //timer
                0x21 => (), //keyboard
                0x2B => pci_handle(0xB),
                0x2C => (), //mouse
                0x2E => (), //disk
                _ => {
                    d("I: ");
                    dh(interrupt as usize);
                    dl();
                }
            }

            loop {
                let status = inb(0x64);
                if status & 0x21 == 1 {
                    let key_event = keyboard_interrupt();
                    if key_event.scancode > 0 {
                        session.on_key(key_event);
                    }
                }else if status & 0x21 == 0x21 {
                    let mouse_event = mouse_interrupt();

                    if mouse_event.valid {
                        session.on_mouse(mouse_event);
                    }
                }else{
                    break;
                }
            }

            session.redraw();

            if interrupt >= 0x20 && interrupt < 0x30 {
                if interrupt >= 0x28 {
                    outb(0xA0, 0x20);
                }

                outb(0x20, 0x20);
            }

            asm!("sti\n
                hlt\n
                cli");
        }
    }
}