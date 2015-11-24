use redox::Box;
use redox::fs::File;
use redox::io::*;
use redox::mem;
use redox::slice;
use redox::syscall::sys_yield;
use redox::String;
use redox::ToString;
use redox::to_num::ToNum;
use redox::Vec;

use super::Event;
use super::Color;

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
    /// The input scheme
    file: File,
    /// Font file
    font: Vec<u8>,
    /// Window data
    data: Vec<u32>,
}

impl Window {
    /// Create a new window
    pub fn new(x: isize, y: isize, w: usize, h: usize, title: &str) -> Option<Box<Self>> {
        let mut font = Vec::new();
        if let Some(mut font_file) = File::open("file:/ui/unifont.font") {
            font_file.read_to_end(&mut font);
        }

        match File::open(&format!("orbital:///{}/{}/{}/{}/{}", x, y, w, h, title)) {
            Some(file) => Some(box Window {
                x: x,
                y: y,
                w: w,
                h: h,
                t: title.to_string(),
                file: file,
                font: font,
                data: vec![0; w * h * 4],
            }),
            None => None
        }
    }

    //TODO: Replace with smarter mechanism, maybe a move event?
    pub fn sync_path(&mut self) {
        if let Some(path) = self.file.path() {
            //orbital://x/y/w/h/t
            let parts: Vec<&str> = path.split('/').collect();
            if let Some(x) = parts.get(3) {
                self.x = x.to_num_signed();
            }
            if let Some(y) = parts.get(4) {
                self.y = y.to_num_signed();
            }
            if let Some(w) = parts.get(5) {
                self.w = w.to_num();
            }
            if let Some(h) = parts.get(6) {
                self.h = h.to_num();
            }
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
    pub fn set_title(&mut self, _: &str) {
        //TODO
    }

    /// Draw a pixel
    pub fn pixel(&mut self, x: isize, y: isize, color: Color) {
        if x >= 0 && y >= 0 && x < self.w as isize && y < self.h as isize {
            let offset = y as usize * self.w + x as usize;
            self.data[offset] = color.data;
        }
    }

    /// Draw a character, using the loaded font
    pub fn char(&mut self, x: isize, y: isize, c: char, color: Color) {
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

    //TODO move, resize, set_title

    /// Set entire window to a color
    // TODO: Improve speed
    #[allow(unused_variables)]
    pub fn set(&mut self, color: Color) {
        let w = self.w;
        let h = self.h;
        self.rect(0, 0, w, h, color);
    }

    /// Draw rectangle
    // TODO: Improve speed
    #[allow(unused_variables)]
    pub fn rect(&mut self, start_x: isize, start_y: isize, w: usize, h: usize, color: Color) {
        for y in start_y..start_y + h as isize {
            for x in start_x..start_x + w as isize {
                self.pixel(x, y, color);
            }
        }
    }

    /// Display an image
    //TODO: Improve speed
    pub fn image(&mut self, start_x: isize, start_y: isize, w: usize, h: usize, data: &[Color]) {
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
        let mut event = Event::new();
        let event_ptr: *mut Event = &mut event;
        loop {
            match self.file.read(&mut unsafe {
                slice::from_raw_parts_mut(event_ptr as *mut u8, mem::size_of::<Event>())
            }) {
                Some(0) => unsafe { sys_yield() },
                Some(_) => return Some(event),
                None => return None,
            }
        }
    }

    /// Flip the window buffer
    pub fn sync(&mut self) -> bool {
        self.file.seek(SeekFrom::Start(0));
        self.file.write(& unsafe {
            slice::from_raw_parts(self.data.as_ptr() as *const u8, self.data.len() * mem::size_of::<u32>())
        });
        return self.file.sync();
    }

    /// Return a iterator over events
    pub fn event_iter<'a>(&'a mut self) -> EventIter<'a> {
        EventIter {
            window: self,
        }
    }
}

/// Event iterator
pub struct EventIter<'a> {
    window: &'a mut Window,
}

impl<'a> Iterator for EventIter<'a> {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        self.window.poll()
    }
}
