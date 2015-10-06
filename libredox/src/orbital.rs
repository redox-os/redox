use core::mem::size_of;
use core::ops::DerefMut;
use core::slice;

use collections::string::*;

use event::*;

use file::*;

pub struct Window {
    x: isize,
    y: isize,
    w: usize,
    h: usize,
    t: String,
    file: File,
    /// Font file, mut to allow changes
    pub font: File,
}

impl Window {
    /// Create a new window
    pub fn new(x: isize, y: isize, w: usize, h: usize, title: &str) -> Self {
        Window {
            x: x,
            y: y,
            w: w,
            h: h,
            t: title.to_string(),
            file: File::open(&format!("window:///{}/{}/{}/{}/{}", x, y, w, h, title)),
            font: File::open(&"file:///ui/unifont.font".to_string()),
        }
    }

    /// Get x
    //TODO: Sync with window movements
    pub fn x(&self) -> isize {
        self.x
    }

    /// Get y
    //TODO: Sync with window movements
    pub fn y(&self) -> isize {
        self.y
    }

    /// Get width
    pub fn width(&self) -> usize {
        self.w
    }

    /// Get height
    pub fn height(&self) -> usize {
        self.h
    }

    /// Get title
    pub fn title(&self) -> String {
        self.t.clone()
    }

    /// Set title
    pub fn set_title(&mut self, title: &str) {
        //TODO
    }

    /// Draw a pixel
    pub fn pixel(&mut self, x: isize, y: isize, color: [u8; 4]) {
        if x >= 0 && y >= 0 {
            self.file.seek(Seek::Start((y as usize * self.w + x as usize) * 4));
            self.file.write(&color);
        }
    }

    /// Draw a character, using the loaded font
    pub fn char(&mut self, x: isize, y: isize, character: char, color: [u8; 4]) {
        self.font.seek(Seek::Start((character as usize) * 16));
        let mut bitmap: [u8; 16] = [0; 16];
        self.font.read(&mut bitmap);
        for row in 0..16 {
            let row_data = bitmap[row];
            for col in 0..8 {
                let pixel = (row_data >> (7 - col)) & 1;
                if pixel > 0 {
                    self.pixel(x + col as isize, y + row as isize, color);
                }
            }
        }
    }

    //TODO move, resize, setTitle

    /// Set entire window to a color
    //TODO: Improve speed
    #[allow(unused_variables)]
    pub fn set(&mut self, color: [u8; 4]) {
        self.file.seek(Seek::Start(0));
        for y in 0..self.h {
            for x in 0..self.w {
                self.file.write(&color);
            }
        }
    }

    /// Draw rectangle
    //TODO: Improve speed
    #[allow(unused_variables)]
    pub fn rect(&mut self, start_x: isize, start_y: isize, w: usize, h: usize, color: [u8; 4]) {
        for y in start_y..start_y + h as isize {
            for x in start_x..start_x + w as isize {
                self.pixel(x, y, color);
            }
        }
    }

    /// Display an image
    //TODO: Improve speed
    pub fn image(&mut self, start_x: isize, start_y: isize, w: usize, h: usize, data: &[[u8; 4]]){
        let mut i = 0;
        for y in start_y..start_y + h as isize {
            for x in start_x..start_x + w as isize {
                if i < data.len() {
                    self.pixel(x, y, data[i])
                }
                i += 1;
            }
        }
    }

    /// Poll for an event
    //TODO: clean this up
    pub fn poll(&mut self) -> Option<Event> {
        let mut event = box Event::new();
        let event_ptr: *mut Event = event.deref_mut();
        match self.file.read(&mut unsafe { slice::from_raw_parts_mut(event_ptr as *mut u8, size_of::<Event>()) }) {
            Option::Some(_) => return Option::Some(*event),
            Option::None => return Option::None
        }
    }

    /// Flip the window buffer
    pub fn sync(&mut self) -> bool {
        return self.file.sync();
    }
}
