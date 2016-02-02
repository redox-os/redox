use std::fs::File;
use std::io::{Result, Read, Write};
use std::mem;
use std::slice;

use super::{Color, Event, Image, ImageRoi};

pub struct Display {
    file: File,
    image: Image,
}

impl Display {
    pub fn new() -> Result<Display> {
        let file = try!(File::open("display:"));

        let path = try!(file.path()).to_string();
        let res = path.split(":").nth(1).unwrap_or("");
        let width = res.split("/").nth(0).unwrap_or("").parse::<i32>().unwrap_or(0);
        let height = res.split("/").nth(1).unwrap_or("").parse::<i32>().unwrap_or(0);

        Ok(Display {
            file: file,
            image: Image::new(width, height)
        })
    }

    pub fn width(&self) -> i32 {
        self.image.width()
    }

    pub fn height(&self) -> i32 {
        self.image.height()
    }

    pub fn as_roi(&mut self) -> ImageRoi {
        self.image.as_roi()
    }

    pub fn roi(&mut self, x: i32, y: i32, w: i32, h: i32) -> ImageRoi {
        self.image.roi(x, y, w, h)
    }

    /// Return a iterator over events
    pub fn events(&mut self) -> EventIter {
        let mut iter = EventIter {
            events: [Event::new(); 128],
            i: 0,
            count: 0,
        };

        match self.file.read(unsafe {
            slice::from_raw_parts_mut(iter.events.as_mut_ptr() as *mut u8, iter.events.len() * mem::size_of::<Event>())
        }){
            Ok(count) => iter.count = count/mem::size_of::<Event>(),
            Err(_) => (),
        }

        iter
    }

    pub fn flip(&mut self) {
        let data = self.image.data();
        self.file.write(unsafe { slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * mem::size_of::<Color>()) });
    }
}

/// Event iterator
pub struct EventIter {
    events: [Event; 128],
    i: usize,
    count: usize,
}

impl Iterator for EventIter {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        if self.i < self.count {
            if let Some(event) = self.events.get(self.i) {
                self.i += 1;
                Some(*event)
            } else {
                None
            }
        } else {
            None
        }
    }
}
