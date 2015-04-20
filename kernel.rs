#![feature(asm)]
#![feature(core)]
#![feature(intrinsics)]
#![feature(lang_items)]
#![feature(no_std)]
#![feature(start)]
#![no_std]

extern crate core;

use common::debug::*;
use common::memory::*;
use common::string::*;

use drivers::disk::*;
use drivers::keyboard::*;
use drivers::mouse::*;

use filesystems::unfs::*;

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
	pub mod color;
	pub mod display;
	pub mod point;
	pub mod size;
	pub mod window;
}

static mut keyboard_window: Window<'static> = Window{
	point:Point{ x:100, y:100 },
	size:Size { width:640, height:480 },
	title:"Press a function key to load a file"
};

static mut mouse_point: Point = Point {
	x: 16,
	y: 16
};

unsafe fn clear_editor(){
    keyboard_window.title = "Press a function key to load a file";
    TEXT_BUFFER_OFFSET = 0;
    for i in 0..TEXT_BUFFER_SIZE {
        *((TEXT_BUFFER_LOCATION + i*4) as *mut char) = '\0';
    }
}

const TEXT_BUFFER_LOCATION: u32 = 0x180000;
const TEXT_BUFFER_SIZE: u32 = 2000;
static mut TEXT_BUFFER_OFFSET: u32 = 0x0;
unsafe fn load_editor_file(filename: &'static str){
    clear_editor();
    let unfs = UnFS::new(Disk::new());
    let dest = unfs.load(filename);
    if dest > 0 {
        TEXT_BUFFER_OFFSET = 0;
        keyboard_window.title = filename;
        for i in 0..TEXT_BUFFER_SIZE {
            let c = *((dest + i) as *const u8) as char;
            *((TEXT_BUFFER_LOCATION + i*4) as *mut char) = c;
            if c == '\0' {
                TEXT_BUFFER_OFFSET = i;
                break;
            }
        }
        unalloc(dest);
    }else{
        d("Did not find '");
        d(filename);
        d("'\n");
    }
}

unsafe fn process_keyboard_event(keyboard_event: KeyEvent){
    if keyboard_event.pressed {
        match keyboard_event.scancode {
            0x3B => load_editor_file("Test"),
            0x3C => load_editor_file("Test 2"),
            0x3D => load_editor_file("Test 3"),
            _ => ()
        }
        
        match keyboard_event.character {
            '\x00' => (),
            '\x08' => if TEXT_BUFFER_OFFSET > 0 {
                TEXT_BUFFER_OFFSET -= 1;
                *((TEXT_BUFFER_LOCATION + TEXT_BUFFER_OFFSET * 4) as *mut char) = '\0';
            },
            '\x1B' => clear_editor(),
            _ => if TEXT_BUFFER_OFFSET < TEXT_BUFFER_SIZE {
                *((TEXT_BUFFER_LOCATION + TEXT_BUFFER_OFFSET * 4) as *mut char) = keyboard_event.character;
                TEXT_BUFFER_OFFSET += 1;
                *((TEXT_BUFFER_LOCATION + TEXT_BUFFER_OFFSET * 4) as *mut char) = '\0';
            }
        }
    }
}


static mut dragging: bool = false;
static mut drag_point: Point = Point { x:-1, y:-1 };
static mut last_drag_point: Point = Point { x:-1, y:-1 };
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

    if mouse_event.left_button {
        dragging = true;
    }else{
        dragging = false;
    }

    if dragging {
        drag_point = mouse_point;
        if last_drag_point.x >= keyboard_window.point.x
            && last_drag_point.x < keyboard_window.point.x + keyboard_window.size.width as i32
            && last_drag_point.y >= keyboard_window.point.y - 16
            && last_drag_point.y < keyboard_window.point.y
        {
            keyboard_window.point.x += drag_point.x - last_drag_point.x;
            keyboard_window.point.y += drag_point.y - last_drag_point.y;
        }
    }else{
        drag_point = Point { x:-1, y:-1 };
    }

    last_drag_point = drag_point;

}


fn draw() {
	let display = Display::new();
	display.clear(Color::new(64, 64, 64));
	display.rect(Point::new(0, 0), Size::new(display.size().width, 18), Color::new(0, 0, 0));
	display.text(Point::new(display.size().width as i32/ 2 - 3*8, 1), "UberOS", Color::new(255, 255, 255));

	unsafe{
		display.window(&keyboard_window);
		
		let mut row = 0;
		let mut col = 0;
        for i in 0..TEXT_BUFFER_SIZE {
            let c = *((TEXT_BUFFER_LOCATION + i*4) as *const char);
            if c == '\0' {
                break;
            }else if c == '\n' {
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
        }
        
        if col < keyboard_window.size.width / 8 && row < keyboard_window.size.height / 16 {
            display.char(Point::new(keyboard_window.point.x + 8*col as i32, keyboard_window.point.y + 16*row as i32), '_', Color::new(0, 0, 0));
        }

		display.char_bitmap(mouse_point, &MOUSE_CURSOR as *const u8, Color::new(255, 255, 255));
	}

	display.copy();
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
    let a = String::from_str("Test string\n") + String::from_str("Another string\n");
    a.d();
}

unsafe fn initialize(){
    d("Clusters\n");
    cluster_init();
    
    mem_test();
    
    str_test();

    d("Text Buffer\n");
    clear_editor();
    
    d("Keyboard Status\n");
    keyboard_status.lshift = false;
    keyboard_status.rshift = false;
    keyboard_status.caps_lock = false;
    keyboard_status.caps_lock_toggle = false;
    
    d("Mouse Point\n");
    mouse_point.x = 16;
    mouse_point.y = 16;
    
    d("Window Dragging\n");
    dragging = false;
    drag_point.x = -1;
    drag_point.y = -1;
    last_drag_point.x = -1;
    last_drag_point.y = -1;
    
    d("Fonts\n");
    let unfs = UnFS::new(Disk::new());
    let font_location = unfs.load("Font");
    
    if font_location > 0 {
        d("Read font file\n");
        *(FONTLOCATION as *mut u32) = font_location;
        dh(FONTLOCATION);
        d("\n");
        dh(*(FONTLOCATION as *const u32));
        d("\n");
    }else{
        d("Did not find font file\n");
    }
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
			d("Keyboard: ");
			dc(keyboard_event.character);
			d(", ");
			dbh(keyboard_event.scancode);
			if keyboard_event.pressed {
                d(", Pressed\n");
			}else{
                d(", Released\n");
            }
			
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
