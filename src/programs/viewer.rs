use common::memory::*;

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

use programs::program::*;

pub struct Viewer {
    window: Window,
    background: BMP
}

impl Viewer {
    pub unsafe fn new() -> Viewer {
        Viewer {
            window: Window{
                point: Point::new(180, 50),
                size: Size::new(640, 480),
                title: "Press a function key to load a file",
                title_color: Color::new(255, 255, 255),
                border_color: Color::new(0, 0, 0),
                content_color: Color::alpha(0, 0, 0, 0),
                shaded: false,
                dragging: false,
                last_mouse_point: Point::new(0, 0),
                last_mouse_event: MouseEvent {
                    x: 0,
                    y: 0,
                    left_button: false,
                    right_button: false,
                    middle_button: false,
                    valid: false
                }
            },
            background: BMP::new()
        }
    }

    unsafe fn clear(&mut self){
        self.window.title = "Press a function key to load a file";
        self.background = BMP::new();
    }
    
    unsafe fn load(&mut self, filename: &'static str){
        self.window.title = filename;
        let unfs = UnFS::new(Disk::new());
        let background_data = unfs.load(filename);
        self.background = BMP::from_data(background_data);
        self.window.size = self.background.size;
        unalloc(background_data);
    }
}

impl Program for Viewer {
    unsafe fn draw(&self, display: &Display){
        self.window.draw(display);
		
		if ! self.window.shaded {
            // TODO: Improve speed!
            if ! self.window.shaded {
                for y in 0..self.background.size.height {
                    for x in 0..self.background.size.width {
                        display.pixel(Point::new(self.window.point.x + x as i32, self.window.point.y + y as i32), self.background.pixel(Point::new(x as i32, y as i32)));
                    }
                }
            }
        }
    }
    
    unsafe fn on_key(&mut self, key_event: KeyEvent){
        if key_event.pressed {
            match key_event.scancode {
                0x3B => self.load("bmw.bmp"),
                0x3C => self.load("schemabanner.bmp"),
                _ => ()
            }
            
            match key_event.character {
                '\x1B' => self.clear(),
                _ => ()
            }
        }
    }
    
    unsafe fn on_mouse(&mut self, mouse_point: Point, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        return self.window.on_mouse(mouse_point, mouse_event, allow_catch);
    }
}