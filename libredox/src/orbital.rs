use common::event::*;
use common::string::*;

use file::*;

pub struct NewWindow {
    x: isize,
    y: isize,
    w: usize,
    h: usize,
    t: String,
    file: File,
}

impl NewWindow {
    /// Create a new window
    pub fn new(x: isize, y: isize, w: usize, h: usize, title: &String) -> NewWindow {
        NewWindow {
            x: x,
            y: y,
            w: w,
            h: h,
            t: title.clone(),
            file: File::open(&("window://".to_string()
                        + '/' + x + '/' + y + '/' + w + '/' + h
                        + '/' + title)),
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

    /// Draw a pixel
    pub fn pixel(&mut self, x: isize, y: isize, color: [u8; 4]) {
        if x >= 0 && y >= 0 {
            self.file.seek(Seek::Start((y as usize * self.w + x as usize) * 4));
            self.file.write(&color);
        }
    }

    /// Set entire window to a color
    //TODO move, resize, setTitle
    #[allow(unused_variables)]
    pub fn set(&mut self, color: [u8; 4]) {
        self.file.seek(Seek::Start(0));
        for y in 0..self.h {
            for x in 0..self.w {
                self.file.write(&color);
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
    pub fn poll(&mut self) -> Option<Event> {
        let mut event_slice = Event::slice();
        match self.file.read(&mut event_slice) {
            Option::Some(_) => return Option::Some(Event::from_slice(&event_slice)),
            Option::None => return Option::None
        }
    }

    /// Flip the window buffer
    pub fn sync(&mut self) -> bool {
        return self.file.sync();
    }
}
