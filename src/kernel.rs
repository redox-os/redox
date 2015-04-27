#![feature(asm)]
#![feature(box_syntax)]
#![feature(core)]
#![feature(lang_items)]
#![feature(no_std)]
#![no_std]

extern crate core;

use core::mem::size_of;
use core::result::Result;

use common::debug::*;
use common::elf::*;
use common::memory::*;
use common::vector::*;

use drivers::disk::*;
use drivers::keyboard::*;
use drivers::mouse::*;

use filesystems::unfs::*;

use graphics::color::*;
use graphics::display::*;
use graphics::point::*;
use graphics::size::*;

use programs::program::*;
use programs::editor::*;
use programs::viewer::*;

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

mod programs {
    pub mod program;
    pub mod editor;
    pub mod viewer;
}

#[lang = "owned_box"]
pub struct Box<T>(*mut T);

#[lang="exchange_malloc"]
pub fn exchange_malloc(size: usize, align: usize) -> *mut u8{
    alloc(size) as *mut u8
}

#[lang="exchange_free"]
pub fn exchange_free(ptr: *mut u8, size: usize, align: usize){
    unalloc(ptr as usize);
}

unsafe fn initialize(){
    dd(size_of::<usize>() * 8);
    d(" bits");
    dl();

    d("Paging\n");
    page_init();
    
    d("Clusters\n");
    cluster_init();
    
    d("Mouse\n");
    mouse_init();
    
    d("Keyboard Status\n");
    keyboard_init();
    
    d("Fonts\n");
    let unfs = UnFS::new(Disk::new());
    FONT_LOCATION = unfs.load("unifont.font");
    
    if FONT_LOCATION > 0 {
        d("Read font file\n");
    }else{
        d("Did not find font file\n");
    }
    
    d("ELF\n");
    let elf = ELF::new(unfs.load("test.bin"));
    elf.run();
}

const INTERRUPT: *mut u8 = 0x200000 as *mut u8;

#[no_mangle]
pub unsafe fn kernel() {
    if *INTERRUPT == 255 {
        initialize();
        
        let display = Display::new();
        
        let mut mouse_point: Point = Point {
            x: 0,
            y: 0
        };
        
        let programs = 
            Vector::<Box<Program>>::from_value(box Viewer::new()) +
            Vector::<Box<Program>>::from_value(box Editor::new());
        
        loop{
            asm!("cli");
            let interrupt = *INTERRUPT;
            *INTERRUPT = 0;
            
            let mut draw = false;
            match interrupt {
                32 => {
                },
                33 => {
                    let key_event = keyboard_interrupt();
                    
                    d("KEY\n");
                
                    for program in programs.as_slice() {                                            
                        (*program).on_key(key_event);
                    }
    
                    draw = true;
                },
                44 => {
                    let mouse_event = mouse_interrupt();

                    if mouse_event.valid {
                        d("MOUSE\n");
                        mouse_point.x += mouse_event.x;
                        if mouse_point.x < 0 {
                            mouse_point.x = 0;
                        }
                        if mouse_point.x >= display.size().width as i32 {
                            mouse_point.x = display.size().width as i32 - 1;
                        }

                        mouse_point.y += mouse_event.y;
                        if mouse_point.y < 0 {
                            mouse_point.y = 0;
                        }
                        if mouse_point.y >= display.size().height as i32 {
                            mouse_point.y = display.size().height as i32 - 1;
                        }
                        
                        for program in programs.as_slice() {
                            if (*program).on_mouse(mouse_point, mouse_event) {
                                break;
                            }
                        }
                    
                        draw = true;
                    }else{
                        d("INVALID MOUSE\n");
                    }
                },
                255 => {
                    d("INIT\n");
                    draw = true;
                },
                _ => {
                    d("I: ");
                    dd(interrupt as usize);
                    dl();
                }
            }
            
            if draw {
                display.clear(Color::new(64, 64, 64));
        
                display.rect(Point::new(0, 0), Size::new(display.size().width, 18), Color::new(0, 0, 0));
                display.text(Point::new(display.size().width as i32/ 2 - 3*8, 1), "UberOS", Color::new(255, 255, 255));

                
                for program in programs.as_slice() {
                    (*program).draw(&display);
                }
                
                display.char_bitmap(mouse_point, &MOUSE_CURSOR as *const u8, Color::new(255, 255, 255));
                
                display.copy();
            }
            
            asm!("sti\n
                hlt");
        }
	}
}