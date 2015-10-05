use common::event::*;
use common::string::*;

use file::*;
use orbital::*;


struct Font {
    font_file: File,
}

impl Font {

    pub fn new(&self, font_location: &String) {
        Font {
            font_file: File::open(&("file://".to_string()
                             + "/" + font_location.clone())),
        }
    }

    pub fn char(&self, win: Window, point: Point, character: char, color: Color) {
        self.font_file.seek(Seek::Start((character as usize) * 16));
        let bitmap: [u8:16] = [0; 1];
        self.font_file.read(&bitmap);
        for row in 0..16 {
            let row_data = *((bitmap + row) as *const u8);
            for col in 0..8 {
                let pixel = (row_data >> (7 - col)) & 1;
                if pixel > 0 {
                    win.pixel(point.x, point.y, color);
                }
            }
        }
    }
                    
}
