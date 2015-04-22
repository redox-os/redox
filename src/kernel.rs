#![feature(asm)]
#![feature(core)]
#![feature(intrinsics)]
#![feature(lang_items)]
#![feature(no_std)]
#![feature(start)]
#![no_std]

extern crate core;

use core::mem;

use common::debug::*;
use common::memory::*;
use common::string::*;
use common::vector::*;

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

static mut keyboard_window: Window<'static> = Window{
	point: Point{ x:100, y:100 },
	size: Size { width:800, height:600 },
	title: "Press a function key to load a file",
	shaded: false,
	focused: false,
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

static mut image_window: Window<'static> = Window{
	point: Point{ x:50, y:50 },
	size: Size { width:800, height:600 },
	title: "Press a function key to load an image",
	shaded: false,
	focused: false,
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
static mut edit_offset: u32 = 0;

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
    
    if keyboard_window.focused {
        if ! keyboard_window.on_mouse(mouse_point, mouse_event) {
            if image_window.on_mouse(mouse_point, mouse_event) {
                keyboard_window.focused = false;
                image_window.focused = true;
            }
        }
    }else if image_window.focused {
        if ! image_window.on_mouse(mouse_point, mouse_event) {
            if keyboard_window.on_mouse(mouse_point, mouse_event) {
                keyboard_window.focused = true;
                image_window.focused = false;
            }
        }
    }else{
        keyboard_window.focused = true;
        image_window.focused = false;
    }
}

static mut background: BMP = BMP { data: 0, size: Size { width: 0, height: 0 } };
fn draw() {
	unsafe{
        let display = Display::new();
        display.clear(Color::new(64, 64, 64));
        
        display.rect(Point::new(0, 0), Size::new(display.size().width, 18), Color::new(0, 0, 0));
        display.text(Point::new(display.size().width as i32/ 2 - 3*8, 1), "UberOS", Color::new(255, 255, 255));

        if ! image_window.focused {
            display.window(&image_window);
            // TODO: Improve speed!
            if ! image_window.shaded {
                for y in 0..background.size.height {
                    for x in 0..background.size.width {
                        display.pixel(Point::new(image_window.point.x + (x + (image_window.size.width - background.size.width) / 2) as i32, image_window.point.y + (y + (image_window.size.height - background.size.height) / 2) as i32), background.pixel(Point::new(x as i32, y as i32)));
                    }
                }
            }
        }
        
		display.window(&keyboard_window);
		
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
        
        if image_window.focused {
            display.window(&image_window);
            // TODO: Improve speed!
            if ! image_window.shaded {
                for y in 0..background.size.height {
                    for x in 0..background.size.width {
                        display.pixel(Point::new(image_window.point.x + (x + (image_window.size.width - background.size.width) / 2) as i32, image_window.point.y + (y + (image_window.size.height - background.size.height) / 2) as i32), background.pixel(Point::new(x as i32, y as i32)));
                    }
                }
            }
        }

		display.char_bitmap(mouse_point, &MOUSE_CURSOR as *const u8, Color::new(255, 255, 255));
		
        display.copy();
	}
}

fn mem_test(){
    d("A: ");
    let a = alloc(100);
    dh(a);
    dl();
    
    d("B: ");
    let b = alloc(1024*1024 + 1);
    dh(b);
    dl();
    
    d("C: ");
    let c = alloc(2);
    dh(c);
    dl();
    
    d("-B\n");
    unalloc(b);
    
    d("E: ");
    let e = alloc(1024);
    dh(e);
    dl();
    
    d("F: ");
    let f = alloc(1024);
    dh(f);
    dl();
    
    d("G: ");
    let g = alloc(1024);
    dh(g);
    dl();
    
    unalloc(a);
    unalloc(c);
    unalloc(e);
    unalloc(f);
    unalloc(g);
}

fn str_test(){
    let a = String::new() + "Test string: " + 7357 + "\n" +
            "Another string: 0x" + String::from_num_radix(0xDEADBEEF, 16) + "\n";
    a.d();
    a.substr(13, 5).d();
    
    let mut b = Vector::<u32>::new() + 12 + 3 + 5;
    b = b + 2;
    d("Numbers:\n");
    for n_ptr in b.sub(1, 2).as_slice() {
        dd(*n_ptr);
        dl();
    }
    dl();
}

unsafe fn initialize(){
    d("Clusters\n");
    cluster_init();
    
    mem_test();
    
    str_test();

    d("Text Buffer\n");
    edit_string = alloc(mem::size_of::<String>() as u32) as *mut String;
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
        dh(FONT_LOCATION);
        d("\n");
    }else{
        d("Did not find font file\n");
    }
    
    background.drop();
}

const INTERRUPT_LOCATION: u32 = 0x200000;

#[start]
#[no_mangle]
pub fn kernel() {
	let interrupt: u8;
	unsafe {
		interrupt = *(INTERRUPT_LOCATION as *const u8);
	}

	if interrupt == 255 {
        // TODO: Figure out how to fix static mut initialization
        d("Initialize\n");
		unsafe{
            initialize();
            
            draw();
		}
	}else if interrupt == 33 {
		unsafe{
			let keyboard_event = keyboard_interrupt();
			/*
			d("Keyboard: ");
			dc(keyboard_event.character);
			d(", ");
			dbh(keyboard_event.scancode);
			if keyboard_event.pressed {
                d(", Pressed\n");
			}else{
                d(", Released\n");
            }
			*/
			process_keyboard_event(keyboard_event);
			
			draw();
        }
	} else if interrupt == 44 {
		let mouse_event = mouse_interrupt();

		if mouse_event.valid {
			unsafe{
                /*
                d("Mouse: ");
                dd(mouse_point.x as u32);
                d(", ");
                dd(mouse_point.y as u32);
                if mouse_event.left_button {
                    d(", Left");
                }
                if mouse_event.middle_button {
                    d(", Middle");
                }
                if mouse_event.right_button {
                    d(", Right");
                }
                d("\n");
                */
                
				process_mouse_event(mouse_event);
				
				draw();
			}
		}
	}
}

extern "rust-intrinsic" {
	fn offset<T>(dst: *const T, offset: isize) -> *const T;
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: u32) -> *mut u8 {
	let mut i = 0;
	while i < n {
		*(offset(dest as *const u8, i as isize) as *mut u8) =
		*offset(src, i as isize);
		i += 1;
	}
	return dest;
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: u32) -> *mut u8 {
	let mut i = 0;
	while i < n {
		*(offset(s as *const u8, i as isize) as *mut u8) = c as u8;
		i += 1;
	}
	return s;
}
