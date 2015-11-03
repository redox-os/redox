pub mod event;
pub mod window;
/*
use alloc::boxed::Box;

use core::mem;
use core::ops::DerefMut;
use core::slice;

use string::{String, ToString};
use vec::Vec;

use event::*;
use graphics::color::Color;
use fs::File;
use io::*;

use syscall::sys_yield;

pub mod event;

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
        if let Some(mut font_file) = File::open("file:///ui/unifont.font") {
            font_file.read_to_end(&mut font);
        }

        match File::open(&format!("window:///{}/{}/{}/{}/{}", x, y, w, h, title)) {
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
        let mut event = box Event::new();
        let event_ptr: *mut Event = event.deref_mut();
        loop {
            match self.file.read(&mut unsafe {
                slice::from_raw_parts_mut(event_ptr as *mut u8, mem::size_of::<Event>())
            }) {
                Some(0) => unsafe { sys_yield() },
                Some(_) => return Some(*event),
                None => return None,
            }
        }
    }

    /// Flip the window buffer
    pub fn sync(&mut self) -> bool {
        self.file.seek(SeekFrom::Start(0));
        let to_write: &[u8] = unsafe{ mem::transmute::<&[u32],&[u8]>(&self.data) };
        self.file.write(to_write);
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
*/
