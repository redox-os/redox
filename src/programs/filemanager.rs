use common::debug::*;
use common::memory::*;
use common::string::*;
use common::vector::*;

use drivers::disk::*;
use drivers::keyboard::*;
use drivers::mouse::*;

use filesystems::unfs::*;

use graphics::color::*;
use graphics::display::*;
use graphics::point::*;
use graphics::size::*;
use graphics::window::*;

use programs::program::*;

pub struct FileManager {
    window: Window,
    files: Vector<String>
}

impl FileManager {
    pub unsafe fn new() -> FileManager {
        let mut size = Size {
            width: 0,
            height: 0
        };
        
        let files = UnFS::new(Disk::new()).list();
        
        if size.height < files.len() as u32 * 16 {
            size.height = files.len() as u32 * 16;
        }
        
        for file in files.as_slice() {
            if size.width < (file.len() + 1) as u32 * 8 {
                size.width = (file.len() + 1) as u32 * 8 ;
            }
        }
    
        FileManager {
            window: Window{
                point: Point{ x:0, y:50 },
                size: size,
                title: "File Manager",
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
            files: files
        }
    }
}

impl Program for FileManager {
    unsafe fn draw(&self, display: &Display){
        self.window.draw(display);
		
		if ! self.window.shaded {
            let mut row = 0;
            for string in self.files.as_slice() {
                let mut col = 0;
                for c_ptr in string.as_slice() {                
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
                }
                row += 1;
            }
        }
    }
    
    unsafe fn on_key(&mut self, key_event: KeyEvent){
    }
    
    unsafe fn on_mouse(&mut self, mouse_point: Point, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return self.window.on_mouse(mouse_point, mouse_event, allow_catch);
    }
}