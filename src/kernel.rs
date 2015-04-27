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
use common::pio::*;
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

pub unsafe fn timestamp() -> u64 {
    let low: u32;
    let high: u32;
    asm!("rdtsc"
        : "={eax}"(low), "={edx}"(high) : : "intel");
        
    return low as u64;
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
            *INTERRUPT = 255;
            
            let mut draw = false;
            match interrupt {
                32 => {
                },
                33 => {
                    let key_event = keyboard_interrupt();
                    
                    d("KEY ");
                    dc(key_event.character);
                    dl();
                
                    for program in programs.as_slice() {                                            
                        (*program).on_key(key_event);
                    }
                    
                    draw = true;
                },
                44 => {
                    let mouse_event = mouse_interrupt();

                    if mouse_event.valid {                        
                        mouse_point.x += mouse_event.x;
                        if mouse_point.x < 0 {
                            mouse_point.x = 0;
                        }
                        if mouse_point.x >= display.size.width as i32 {
                            mouse_point.x = display.size.width as i32 - 1;
                        }

                        mouse_point.y += mouse_event.y;
                        if mouse_point.y < 0 {
                            mouse_point.y = 0;
                        }
                        if mouse_point.y >= display.size.height as i32 {
                            mouse_point.y = display.size.height as i32 - 1;
                        }
                        
                        d("MOUSE ");
                        dd(mouse_point.x as usize);
                        d(", ");
                        dd(mouse_point.y as usize);
                        dl();
                        
                        for program in programs.as_slice() {
                            if (*program).on_mouse(mouse_point, mouse_event) {
                                break;
                            }
                        }
                        
                        draw = true;
                    }
                },
                255 => {
                    draw = true;
                },
                _ => {
                    d("I: ");
                    dd(interrupt as usize);
                    dl();
                }
            }
            
            if draw {
                let t_clear = timestamp();
                display.clear(Color::new(64, 64, 64));
                
                let t_rect = timestamp();
                display.rect(Point::new(0, 0), Size::new(display.size.width, 18), Color::new(0, 0, 0));
                
                let t_text = timestamp();
                display.text(Point::new(display.size.width as i32/ 2 - 3*8, 1), "UberOS", Color::new(255, 255, 255));
                
                let t_prog = timestamp();
                for program in programs.as_slice() {
                    (*program).draw(&display);
                }
                
                let t_mouse = timestamp();
                display.char_bitmap(mouse_point, &MOUSE_CURSOR as *const u8, Color::new(255, 255, 255));
                
                let t_copy = timestamp();
                display.copy();
                
                let t_finish = timestamp();
                
                /*
                d("Clear: ");
                dd((t_rect - t_clear) as usize);
                dl();
                
                d("Rect: ");
                dd((t_text - t_rect) as usize);
                dl();
                
                d("Text: ");
                dd((t_prog - t_text) as usize);
                dl();
                
                d("Prog: ");
                dd((t_mouse - t_prog) as usize);
                dl();
                
                d("Mouse: ");
                dd((t_copy - t_mouse) as usize);
                dl();
                
                d("Copy: ");
                dd((t_finish - t_copy) as usize);
                dl();
                */
            }
            
            
            if interrupt >= 0x20 && interrupt < 0x30 {
                if interrupt >= 0x28 {
                    outb(0xA0, 0x20);
                }
                
                outb(0x20, 0x20);
            }
            
            asm!("sti\nhlt");
        }
	}
}