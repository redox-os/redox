#![feature(asm)]
#![feature(core)]
#![feature(no_std)]
#![no_std]

extern crate core;

use core::mem::size_of;

use common::debug::*;
use common::elf::*;
use common::memory::*;

use drivers::disk::*;
use drivers::keyboard::*;
use drivers::mouse::*;

use filesystems::unfs::*;

use graphics::color::*;
use graphics::display::*;
use graphics::point::*;
use graphics::size::*;

use programs::editor::*;

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
    pub mod editor;
}

static mut mouse_point: Point = Point {
	x: 16,
	y: 16
};

static mut editor: *mut Editor = 0 as *mut Editor;

unsafe fn process_keyboard_event(key_event: KeyEvent){
    (*editor).on_key(key_event);
}

unsafe fn process_mouse_event(mouse_event: MouseEvent){
    let display = Display::new();

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
    
    (*editor).on_mouse(mouse_point, mouse_event);
}

fn draw() {
	unsafe{
        let display = Display::new();
        display.clear(Color::new(64, 64, 64));
        
        display.rect(Point::new(0, 0), Size::new(display.size().width, 18), Color::new(0, 0, 0));
        display.text(Point::new(display.size().width as i32/ 2 - 3*8, 1), "UberOS", Color::new(255, 255, 255));

        (*editor).draw(&display);
        
		display.char_bitmap(mouse_point, &MOUSE_CURSOR as *const u8, Color::new(255, 255, 255));
		
        display.copy();
	}
}

unsafe fn initialize(){
    dd(size_of::<usize>() * 8);
    d(" bits");
    dl();

    d("Paging\n");
    page_init();
    
    d("Clusters\n");
    cluster_init();
    
    d("Keyboard Status\n");
    keyboard_status.lshift = false;
    keyboard_status.rshift = false;
    keyboard_status.caps_lock = false;
    keyboard_status.caps_lock_toggle = false;
    
    d("Mouse Point\n");
    mouse_point.x = 16;
    mouse_point.y = 16;
    
    d("Fonts\n");
    let unfs = UnFS::new(Disk::new());
    FONT_LOCATION = unfs.load("unifont.font");
    
    if FONT_LOCATION > 0 {
        d("Read font file\n");
    }else{
        d("Did not find font file\n");
    }
    
    d("Editor\n");
    editor = alloc(size_of::<Editor>()) as *mut Editor;
    *editor = Editor::new();
    
    d("ELF\n");
    let elf = ELF::new(unfs.load("test.bin"));
    elf.run();
}

const INTERRUPT_LOCATION: *const u8 = 0x200000 as *const u8;

#[no_mangle]
pub fn kernel() {
	let interrupt: u8;
	unsafe {
		interrupt = *INTERRUPT_LOCATION;
	}

	if interrupt == 255 {
        // TODO: Figure out how to fix static mut initialization
		unsafe{
            initialize();
            
            draw();
		}
	}else if interrupt == 33 {
		unsafe{
			process_keyboard_event(keyboard_interrupt());
			
			draw();
        }
	} else if interrupt == 44 {
		let mouse_event = mouse_interrupt();

		if mouse_event.valid {
			unsafe{
				process_mouse_event(mouse_event);
				
				draw();
			}
		}
	}
}