#![feature(asm)]
#![feature(core)]
#![feature(intrinsics)]
#![feature(lang_items)]
#![feature(no_std)]
#![feature(start)]
#![feature(unboxed_closures)]
#![no_std]

extern crate core;

use core::mem::size_of;

use common::debug::*;
use common::memory::*;
use common::string::*;

use drivers::disk::*;
use drivers::keyboard::*;
use drivers::mouse::*;

use filesystems::unfs::*;

use graphics::bmp::*;
use graphics::color::*;
use graphics::display::*;
use graphics::point::*;
use graphics::size::*;
use graphics::window::*;

mod common {
    pub mod debug;
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

static mut keyboard_window: Window = Window{
	point: Point{ x:100, y:100 },
	size: Size { width:800, height:600 },
	title: "Press a function key to load a file",
	shaded: false,
	dragging: false,
    last_mouse_point: Point {
        x: 0,
        y: 0
    },
    last_mouse_event: MouseEvent {
        x: 0,
        y: 0,
        left_button: false,
        right_button: false,
        middle_button: false,
        valid: false
    }
};

static mut image_window: Window = Window{
	point: Point{ x:50, y:50 },
	size: Size { width:800, height:600 },
	title: "Press a function key to load an image",
	shaded: false,
	dragging: false,
    last_mouse_point: Point {
        x: 0,
        y: 0
    },
    last_mouse_event: MouseEvent {
        x: 0,
        y: 0,
        left_button: false,
        right_button: false,
        middle_button: false,
        valid: false
    }
};

static mut mouse_point: Point = Point {
	x: 16,
	y: 16
};

static mut edit_string: *mut String = 0 as *mut String;
static mut edit_offset: usize = 0;

unsafe fn clear_editor(){
    keyboard_window.title = "Press a function key to load a file";
    *edit_string = String::new();
    edit_offset = 0;
}

unsafe fn load_editor_file(filename: &'static str){
    clear_editor();
    let unfs = UnFS::new(Disk::new());
    let dest = unfs.load(filename);
    if dest > 0 {
        keyboard_window.title = filename;
        *edit_string = String::from_c_str(dest as *const u8);
        edit_offset = (*edit_string).len();
        unalloc(dest);
    }else{
        d("Did not find '");
        d(filename);
        d("'\n");
    }
}

unsafe fn load_background(filename: &'static str){
    let unfs = UnFS::new(Disk::new());
    let background_data = unfs.load(filename);
    background.drop();
    image_window.title = filename;
    background = BMP::new(background_data);
    unalloc(background_data);
}

unsafe fn process_keyboard_event(keyboard_event: KeyEvent){
    if keyboard_event.pressed {
        match keyboard_event.scancode {
            0x3B => load_editor_file("README.md"),
            0x3C => load_editor_file("LICENSE.md"),
            0x3D => load_background("bmw.bmp"),
            0x3E => load_background("stonehenge.bmp"),
            0x3F => load_background("tiger.bmp"),
            0x4B => if edit_offset > 0 {
                        edit_offset -= 1;
                    },
            0x4D => if edit_offset < (*edit_string).len() {
                        edit_offset += 1;
                    },
            _ => ()
        }
        
        match keyboard_event.character {
            '\x00' => (),
            '\x08' => if edit_offset > 0 {
                *edit_string = (*edit_string).substr(0, edit_offset - 1) + (*edit_string).substr(edit_offset, (*edit_string).len() - edit_offset);
                edit_offset -= 1;
            },
            '\x1B' => clear_editor(),
            _ => {
                *edit_string = (*edit_string).substr(0, edit_offset) + keyboard_event.character + (*edit_string).substr(edit_offset, (*edit_string).len() - edit_offset);
                edit_offset += 1;
            }
        }
    }
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
    
    if ! keyboard_window.on_mouse(mouse_point, mouse_event) {
        image_window.on_mouse(mouse_point, mouse_event);
    }
}

static mut background: BMP = BMP { data: 0, size: Size { width: 0, height: 0 } };
fn draw() {
	unsafe{
        let display = Display::new();
        display.clear(Color::new(64, 64, 64));
        
        display.rect(Point::new(0, 0), Size::new(display.size().width, 18), Color::new(0, 0, 0));
        display.text(Point::new(display.size().width as i32/ 2 - 3*8, 1), "UberOS", Color::new(255, 255, 255));

        image_window.draw(&display);
        // TODO: Improve speed!
        if ! image_window.shaded {
            for y in 0..background.size.height {
                for x in 0..background.size.width {
                    display.pixel(Point::new(image_window.point.x + (x + (image_window.size.width - background.size.width) / 2) as i32, image_window.point.y + (y + (image_window.size.height - background.size.height) / 2) as i32), background.pixel(Point::new(x as i32, y as i32)));
                }
            }
        }
        
		keyboard_window.draw(&display);
		
		if ! keyboard_window.shaded {
            let mut offset = 0;
            let mut row = 0;
            let mut col = 0;
            for c_ptr in (*edit_string).as_slice() {
                if offset == edit_offset && col < keyboard_window.size.width / 8 && row < keyboard_window.size.height / 16 {
                    display.char(Point::new(keyboard_window.point.x + 8*col as i32, keyboard_window.point.y + 16*row as i32), '_', Color::new(128, 128, 128));
                }
            
                let c = *c_ptr;
                if c == '\n' {
                    col = 0;
                    row += 1;
                }else if c == '\t' {
                    col += 8 - col % 8;
                }else{
                    if col < keyboard_window.size.width / 8 && row < keyboard_window.size.height / 16 {
                        let point = Point::new(keyboard_window.point.x + 8*col as i32, keyboard_window.point.y + 16*row as i32);
                        display.char(point, c, Color::new(255, 255, 255));
                        col += 1;
                    }
                }
                if col >= keyboard_window.size.width / 8 {
                    col = 0;
                    row += 1;
                }
                
                offset += 1;
            }
            
            if offset == edit_offset && col < keyboard_window.size.width / 8 && row < keyboard_window.size.height / 16 {
                display.char(Point::new(keyboard_window.point.x + 8*col as i32, keyboard_window.point.y + 16*row as i32), '_', Color::new(128, 128, 128));
            }
        }
        
		display.char_bitmap(mouse_point, &MOUSE_CURSOR as *const u8, Color::new(255, 255, 255));
		
        display.copy();
	}
}

unsafe fn initialize(){
    dd(size_of::<usize>() * 8);
    d(" bits");
    dl();

    d("Clusters\n");
    cluster_init();

    d("Text Buffer\n");
    edit_string = alloc(size_of::<String>()) as *mut String;
    *edit_string = String::new();
    clear_editor();
    
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
    FONT_LOCATION = unfs.load("font.unicode.bin");
    
    if FONT_LOCATION > 0 {
        d("Read font file\n");
    }else{
        d("Did not find font file\n");
    }
    
    background.drop();
}

const INTERRUPT_LOCATION: usize = 0x200000;

#[start]
#[no_mangle]
pub fn kernel() {
	let interrupt: u8;
	unsafe {
		interrupt = *(INTERRUPT_LOCATION as *const u8);
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