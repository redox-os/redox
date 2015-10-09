use core::mem;
use core::ops::DerefMut;
use core::slice;

use string::*;
use vec::Vec;

use event::*;

use fs::file::*;
use io::*;

/// A window
pub struct Window {
    /// The x coordinate of the window
    x: isize,
    /// The y coordinate of the window
    y: isize,
    /// The width of the window
    w: usize,
    /// The height of the window
    h: usize,
    /// The title of the window
    t: String,
    file: File,
    /// Font file
    font: Vec<u8>,
    /// Window data
    data: Vec<u8>,
}

impl Window {
    /// Create a new window
    pub fn new(x: isize, y: isize, w: usize, h: usize, title: &str) -> Self {
        let mut font_file = File::open("file:///ui/unifont.font");

        let mut font;
        match font_file.seek(Seek::End(0)) {
            Some(length) => {
                font = vec![0; length];

                font_file.seek(Seek::Start(0));
                font_file.read(&mut font);
            },
            None => font = Vec::new(),
        }

        Window {
            x: x,
            y: y,
            w: w,
            h: h,
            t: title.to_string(),
            file: File::open(&format!("window:///{}/{}/{}/{}/{}", x, y, w, h, title)),
            font: font,
            data: vec![0; w * h * 4],
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
        if x >= 0 && y >= 0 && x < self.w as isize && y < self.h as isize {
            let offset = (y as usize * self.w + x as usize) * 4;
            //TODO: Alpha
            self.data[offset + 0] = color[0];
            self.data[offset + 1] = color[1];
            self.data[offset + 2] = color[2];
            self.data[offset + 3] = color[3];
        }
    }

    /// Draw a character, using the loaded font
    pub fn char(&mut self, x: isize, y: isize, c: char, color: [u8; 4]) {
        let mut offset = (c as usize) * 16;
        for row in 0..16 {
            let row_data;
            if offset < self.font.len() {
                row_data = self.font[offset];
            } else {
                row_data = 0;
            }

            for col in 0..8 {
                let pixel = (row_data >> (7 - col)) & 1;
                if pixel > 0 {
                    self.pixel(x + col as isize, y + row as isize, color);
                }
            }

            offset += 1;
        }
    }

    //TODO move, resize, setTitle

    /// Set entire window to a color
    // TODO: Improve speed
    #[allow(unused_variables)]
    pub fn set(&mut self, color: [u8; 4]) {
        let w = self.w;
        let h = self.h;
        self.rect(0, 0, w, h, color);
    }

    /// Draw rectangle
    // TODO: Improve speed
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
    pub fn image(&mut self, start_x: isize, start_y: isize, w: usize, h: usize, data: &[[u8; 4]]) {
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
        match self.file.read(&mut unsafe {
            slice::from_raw_parts_mut(event_ptr as *mut u8, mem::size_of::<Event>())
        }) {
            Option::Some(_) => return Option::Some(*event),
            Option::None => return Option::None,
        }
    }

    /// Flip the window buffer
    pub fn sync(&mut self) -> bool {
        self.file.seek(Seek::Start(0));
        self.file.write(&self.data);
        return self.file.sync();
    }
}
