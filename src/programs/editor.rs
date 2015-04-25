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

pub struct Editor {
    window: Window,
    string: String,
    offset: usize,
    background: BMP
}

impl Editor {
    pub unsafe fn new() -> Editor {
        Editor {
            window: Window{
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
            },
            string: String::new(),
            offset: 0,
            background: BMP::new()
        }
    }
    
    pub unsafe fn draw(&self, display: &Display){
        self.window.draw(display);
		
		if ! self.window.shaded {
            // TODO: Improve speed!
            if ! self.window.shaded {
                for y in 0..self.background.size.height {
                    for x in 0..self.background.size.width {
                        display.pixel(Point::new(self.window.point.x + (x + (self.window.size.width - self.background.size.width) / 2) as i32, self.window.point.y + (y + (self.window.size.height - self.background.size.height) / 2) as i32), self.background.pixel(Point::new(x as i32, y as i32)));
                    }
                }
            }
		
            let mut offset = 0;
            let mut row = 0;
            let mut col = 0;
            for c_ptr in self.string.as_slice() {
                if offset == self.offset && col < self.window.size.width / 8 && row < self.window.size.height / 16 {
                    display.char(Point::new(self.window.point.x + 8*col as i32, self.window.point.y + 16*row as i32), '_', Color::new(128, 128, 128));
                }
            
                let c = *c_ptr;
                if c == '\n' {
                    col = 0;
                    row += 1;
                }else if c == '\t' {
                    col += 8 - col % 8;
                }else{
                    if col < self.window.size.width / 8 && row < self.window.size.height / 16 {
                        let point = Point::new(self.window.point.x + 8*col as i32, self.window.point.y + 16*row as i32);
                        display.char(point, c, Color::new(255, 255, 255));
                        col += 1;
                    }
                }
                if col >= self.window.size.width / 8 {
                    col = 0;
                    row += 1;
                }
                
                offset += 1;
            }
            
            if offset == self.offset && col < self.window.size.width / 8 && row < self.window.size.height / 16 {
                display.char(Point::new(self.window.point.x + 8*col as i32, self.window.point.y + 16*row as i32), '_', Color::new(128, 128, 128));
            }
        }
    }
    
    pub unsafe fn on_key(&mut self, key_event: KeyEvent){
        if key_event.pressed {
            match key_event.scancode {
                0x3B => self.load("README.md"),
                0x3C => self.load("LICENSE.md"),
                0x3D => self.load_background("bmw.bmp"),
                0x3E => self.load_background("stonehenge.bmp"),
                0x3F => self.load_background("tiger.bmp"),
                0x4B => if self.offset > 0 {
                            self.offset -= 1;
                        },
                0x4D => if self.offset < self.string.len() {
                            self.offset += 1;
                        },
                _ => ()
            }
            
            match key_event.character {
                '\x00' => (),
                '\x08' => if self.offset > 0 {
                    self.string = self.string.substr(0, self.offset - 1) + self.string.substr(self.offset, self.string.len() - self.offset);
                    self.offset -= 1;
                },
                '\x1B' => {
                        self.clear();
                        self.background = BMP::new()
                },
                _ => {
                    self.string = self.string.substr(0, self.offset) + key_event.character + self.string.substr(self.offset, self.string.len() - self.offset);
                    self.offset += 1;
                }
            }
        }
    }
    
    pub unsafe fn on_mouse(&mut self, mouse_point: Point, mouse_event: MouseEvent){
        self.window.on_mouse(mouse_point, mouse_event);
    }

    unsafe fn clear(&mut self){
        self.window.title = "Press a function key to load a file";
        self.string = String::new();
        self.offset = 0;
    }

    unsafe fn load(&mut self, filename: &'static str){
        self.clear();
        let unfs = UnFS::new(Disk::new());
        let dest = unfs.load(filename);
        if dest > 0 {
            self.window.title = filename;
            self.string = String::from_c_str(dest as *const u8);
            self.offset = self.string.len();
            unalloc(dest);
        }else{
            d("Did not find '");
            d(filename);
            d("'\n");
        }
    }
    
    unsafe fn load_background(&mut self, filename: &'static str){
        let unfs = UnFS::new(Disk::new());
        let background_data = unfs.load(filename);
        self.background = BMP::from_data(background_data);
        unalloc(background_data);
    }
}