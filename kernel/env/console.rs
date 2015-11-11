use alloc::boxed::Box;

use collections::String;

use graphics::color::Color;
use graphics::display::Display;
use graphics::point::Point;
use graphics::size::Size;

pub struct Console {
    pub display: Box<Display>,
    pub point: Point,
    pub draw: bool,
    pub redraw: bool,
    pub command: String
}

impl Console {
    pub fn new() -> Box<Console> {
        box Console {
            display: unsafe { Display::root() },
            point: Point::new(0, 0),
            draw: false,
            redraw: true,
            command: String::new()
        }
    }

    pub fn write(&mut self, byte: u8){
        self.display.rect(self.point, Size::new(8, 16), Color::new(0, 0, 0));
        if byte == 10 {
            self.point.x = 0;
            self.point.y += 16;
        } else if byte == 8 {
            // TODO: Fix up hack for backspace
            self.point.x -= 8;
            if self.point.x < 0 {
                self.point.x = 0
            }
            self.display.rect(self.point, Size::new(8, 16), Color::new(0, 0, 0));
        } else {
            self.display.char(self.point, byte as char, Color::new(255, 255, 255));
            self.point.x += 8;
        }
        if self.point.x >= self.display.width as isize {
            self.point.x = 0;
            self.point.y += 16;
        }
        while self.point.y + 16 > self.display.height as isize {
            self.display.scroll(16);
            self.point.y -= 16;
        }
        self.display.rect(self.point, Size::new(8, 16), Color::new(255, 255, 255));
        self.redraw = true;
        // If contexts disabled, probably booting up
        if ! unsafe { ::scheduler::context::context_enabled } && self.draw && self.redraw {
            self.redraw = false;
            self.display.flip();
        }
    }
}
