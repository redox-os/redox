use filesystems::unfs::*;

use graphics::color::*;
use graphics::size::*;
use graphics::window::*;

use programs::common::*;

pub struct FileManager {
    window: Window,
    files: Vec<String>,
    selected: isize
}

impl SessionItem for FileManager {
    fn new() -> FileManager {
        let mut size = Size::new(0, 0);

        let files = UnFS::new().list("".to_string());

        if size.height < files.len() * 16 {
            size.height = files.len() * 16;
        }

        for file in files.iter() {
            if size.width < (file.len() + 1) * 8 {
                size.width = (file.len() + 1) * 8 ;
            }
        }

        let ret = FileManager {
            window: Window{
                point: Point::new(10, 50),
                size: size,
                title: String::from_str("File Manager"),
                title_color: Color::new(0, 0, 0),
                border_color: Color::new(255, 255, 255),
                content_color: Color::alpha(0, 0, 0, 196),
                shaded: false,
                closed: false,
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
            files: files,
            selected: -1
        };

        return ret;
    }

    #[allow(unused_variables)]
    fn draw(&mut self, display: &Display, events: &mut Vec<URL>) -> bool{
        if ! self.window.draw(display) {
            return false;
        }

        if ! self.window.shaded {
            let mut i = 0;
            let mut row = 0;
            for string in self.files.iter() {
                let mut col = 0;
                for c in string.chars() {
                    if c == '\n' {
                        col = 0;
                        row += 1;
                    }else if c == '\t' {
                        col += 8 - col % 8;
                    }else{
                        let color;
                        if i == self.selected {
                            color = Color::new(128, 128, 128);
                        }else{
                            color = Color::new(255, 255, 255);
                        }

                        if col < self.window.size.width / 8 && row < self.window.size.height / 16 {
                            let point = Point::new(self.window.point.x + 8*col as isize, self.window.point.y + 16*row as isize);
                            display.char(point, c, color);
                            col += 1;
                        }
                    }
                    if col >= self.window.size.width / 8 {
                        col = 0;
                        row += 1;
                    }
                }
                row += 1;
                i += 1;
            }
        }

        return true;
    }

    #[allow(unused_variables)]
    fn on_key(&mut self, events: &mut Vec<URL>, key_event: KeyEvent){
        if key_event.pressed {
            match key_event.scancode {
                0x01 => self.selected = -1,
                0x47 => self.selected = 0,
                0x48 => if self.selected > 0 {
                            self.selected -= 1;
                        },
                0x4F => self.selected = self.files.len() as isize - 1,
                0x50 => if self.selected < self.files.len() as isize - 1 {
                            self.selected += 1;
                        },
                _ => ()
            }
            match key_event.character {
                '\0' => (),
                '\n' => {
                    if self.selected >= 0 && self.selected < self.files.len() as isize {
                        match self.files.get(self.selected as usize) {
                            Option::Some(file) => {
                                events.push(URL::from_string("open:///file:///".to_string() + file.clone()));
                            },
                            Option::None => ()
                        }
                    }
                },
                _ => {
                    let mut i = 0;
                    for file in self.files.iter() {
                        if file[0] == key_event.character {
                            self.selected = i;
                            break;
                        }
                        i += 1;
                    }
                }
            }
        }
    }

    #[allow(unused_variables)]
    fn on_mouse(&mut self, events: &mut Vec<URL>, mouse_point: Point, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        if self.window.on_mouse(mouse_point, mouse_event, allow_catch) {
            if ! self.window.shaded {
                let mut i = 0;
                let mut row = 0;
                for file in self.files.iter() {
                    let mut col = 0;
                    for c in file.chars() {
                        if c == '\n' {
                            col = 0;
                            row += 1;
                        }else if c == '\t' {
                            col += 8 - col % 8;
                        }else{
                            if col < self.window.size.width / 8 && row < self.window.size.height / 16 {
                                let point = Point::new(self.window.point.x + 8*col as isize, self.window.point.y + 16*row as isize);
                                if mouse_point.x >= point.x && mouse_point.x < point.x + 8 && mouse_point.y >= point.y && mouse_point.y < point.y + 16 {
                                    self.selected = i;
                                }
                                col += 1;
                            }
                        }
                        if col >= self.window.size.width / 8 {
                            col = 0;
                            row += 1;
                        }
                    }
                    row += 1;
                    i += 1;
                }
            }

            return true;
        }else{
            return false;
        }
    }
}
