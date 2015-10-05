use common::event::*;
use common::string::*;

use file::*;
use orbital::*;


struct Font {
    file: File,
}

impl Font {

    pub fn new(font_location: &String) -> Font {
        Font {
            file: File::open(&("file://".to_string()
                             + "/" + font_location.clone())),
        }
    }

    pub fn char(&mut self, win: &mut NewWindow, x: isize, y: isize, character: char, color: [u8; 4]) {
        self.file.seek(Seek::Start((character as usize) * 16));
        let mut bitmap: [u8; 16] = [0; 16];
        self.file.read(&mut bitmap);
        for row in 0..16 {
            let row_data = bitmap[row];
            for col in 0..8 {
                let pixel = (row_data >> (7 - col)) & 1;
                if pixel > 0 {
                    win.pixel(x + col as isize, y + row as isize, color);
                }
            }
        }
    }
}
