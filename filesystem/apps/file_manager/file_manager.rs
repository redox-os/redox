use redox::*;
use redox::time::*;

pub struct FileManager {
    folder_icon: BMPFile,
    audio_icon: BMPFile,
    bin_icon: BMPFile,
    image_icon: BMPFile,
    source_icon: BMPFile,
    script_icon: BMPFile,
    text_icon: BMPFile,
    file_icon: BMPFile,
    files: Vec<String>,
    selected: isize,
    last_mouse_event: MouseEvent,
    click_time: Duration,
}

fn load_icon(path: &str) -> BMPFile {
    let mut resource = File::open(&("file:///ui/mimetypes/".to_string() + path + ".bmp"));

    let mut vec: Vec<u8> = Vec::new();
    resource.read_to_end(&mut vec);

    BMPFile::from_data(&vec)
}

impl FileManager {
    pub fn new() -> Self {
        FileManager {
            folder_icon: load_icon("inode-directory"),
            audio_icon: load_icon("audio-x-wav"),
            bin_icon: load_icon("application-x-executable"),
            image_icon: load_icon("image-x-generic"),
            source_icon: load_icon("text-x-makefile"),
            script_icon: load_icon("text-x-script"),
            text_icon: load_icon("text-x-generic"),
            file_icon: load_icon("unknown"),
            files: Vec::new(),
            selected: -1,
            last_mouse_event: MouseEvent {
                x: 0,
                y: 0,
                left_button: false,
                middle_button: false,
                right_button: false,
            }
            click_time: Duration::new(0, 0),
        }
    }

    fn draw_content(&mut self, window: &mut Window) {
        window.set([255, 255, 255, 255]);

        let mut i = 0;
        let mut row = 0;
        for string in self.files.iter() {
            if i == self.selected {
                let width = window.width();
                window.rect(0, 32 * row as isize, width, 32, [224, 224, 224, 255]);
            }

            if string.ends_with('/') {
                window.image(0,
                             32 * row as isize,
                             self.folder_icon.width(),
                             self.folder_icon.height(),
                             self.folder_icon.as_slice());
            } else if string.ends_with(".wav") {
                window.image(0,
                             32 * row as isize,
                             self.audio_icon.width(),
                             self.audio_icon.height(),
                             self.audio_icon.as_slice());
            } else if string.ends_with(".bin") {
                window.image(0,
                             32 * row as isize,
                             self.bin_icon.width(),
                             self.bin_icon.height(),
                             self.bin_icon.as_slice());
            } else if string.ends_with(".bmp") {
                window.image(0,
                             32 * row as isize,
                             self.image_icon.width(),
                             self.image_icon.height(),
                             self.image_icon.as_slice());
            } else if string.ends_with(".rs") || string.ends_with(".asm") || string.ends_with(".list") {
                window.image(0,
                             32 * row as isize,
                             self.source_icon.width(),
                             self.source_icon.height(),
                             self.source_icon.as_slice());
            } else if string.ends_with(".sh") || string.ends_with(".lua") {
                window.image(0,
                             32 * row as isize,
                             self.script_icon.width(),
                             self.script_icon.height(),
                             self.script_icon.as_slice());
            } else if string.ends_with(".md") || string.ends_with(".txt") {
                window.image(0,
                             32 * row as isize,
                             self.text_icon.width(),
                             self.text_icon.height(),
                             self.text_icon.as_slice());
            } else {
                window.image(0,
                             32 * row as isize,
                             self.file_icon.width(),
                             self.file_icon.height(),
                             self.file_icon.as_slice());
            }

            let mut col = 0;
            for c in string.chars() {
                if c == '\n' {
                    col = 0;
                    row += 1;
                } else if c == '\t' {
                    col += 8 - col % 8;
                } else {
                    if col < window.width() / 8 && row < window.height() / 32 {
                        window.char(8 * col as isize + 40,
                                    32 * row as isize + 8,
                                    c,
                                    [0, 0, 0, 255]);
                        col += 1;
                    }
                }
                if col >= window.width() / 8 {
                    col = 0;
                    row += 1;
                }
            }
            row += 1;
            i += 1;
        }

        window.sync();
    }

    fn main(&mut self, path: &str) {
        let mut width = 160;
        let mut height = 0;
        {
            let mut resource = File::open(path);

            let mut vec: Vec<u8> = Vec::new();
            resource.read_to_end(&mut vec);

            for file in unsafe { String::from_utf8_unchecked(vec) }.split('\n') {
                if width < 40 + (file.len() + 1) * 8 {
                    width = 40 + (file.len() + 1) * 8;
                }
                self.files.push(file.to_string());
            }

            if height < self.files.len() * 32 {
                height = self.files.len() * 32;
            }
        }

        let mut window = Window::new((rand() % 400 + 50) as isize,
                                     (rand() % 300 + 50) as isize,
                                     width,
                                     height,
                                     &path);

        self.draw_content(&mut window);

        while let Option::Some(event) = window.poll() {
            match event.to_option() {
                EventOption::Key(key_event) => {
                    if key_event.pressed {
                        match key_event.scancode {
                            K_ESC => break,
                            K_HOME => self.selected = 0,
                            K_UP => if self.selected > 0 {
                                self.selected -= 1;
                            },
                            K_END => self.selected = self.files.len() as isize - 1,
                            K_DOWN => if self.selected < self.files.len() as isize - 1 {
                                self.selected += 1;
                            },
                            _ => match key_event.character {
                                '\0' => (),
                                '\n' => {
                                    if self.selected >= 0 &&
                                       self.selected < self.files.len() as isize {
                                        match self.files.get(self.selected as usize) {
                                            Option::Some(file) => OpenEvent {
                                                url_string: path.to_string() + &file,
                                            }.trigger(),
                                            Option::None => (),
                                        }
                                    }
                                }
                                _ => {
                                    let mut i = 0;
                                    for file in self.files.iter() {
                                        if file.starts_with(key_event.character) {
                                            self.selected = i;
                                            break;
                                        }
                                        i += 1;
                                    }
                                }
                            },
                        }

                        self.draw_content(&mut window);
                    }
                }
                EventOption::Mouse(mouse_event) => {
                    let mut redraw = false;
                    let mut i = 0;
                    let mut row = 0;
                    for file in self.files.iter() {
                        let mut col = 0;
                        for c in file.chars() {
                            if mouse_event.y >= 32 * row as isize &&
                               mouse_event.y < 32 * row as isize + 32 {
                                let click_time = Duration::realtime();
                                if self.selected == i {
                                    if click_time - self.click_time < Duration::new(0, 500 * NANOS_PER_MILLI) {
                                        match self.files.get(self.selected as usize) {
                                            Option::Some(file) => OpenEvent {
                                                url_string: path.to_string() + &file,
                                            }.trigger(),
                                            Option::None => (),
                                        }
                                        self.click_time = Duration::new(0, 0);
                                    }
                                } else {
                                    self.selected = i;
                                    self.click_time = click_time;
                                }
                                redraw = true;
                            }

                            if c == '\n' {
                                col = 0;
                                row += 1;
                            } else if c == '\t' {
                                col += 8 - col % 8;
                            } else {
                                if col < window.width() / 8 && row < window.height() / 32 {
                                    col += 1;
                                }
                            }
                            if col >= window.width() / 8 {
                                col = 0;
                                row += 1;
                            }
                        }
                        row += 1;
                        i += 1;
                    }

                    if redraw {
                        self.draw_content(&mut window);
                    }

                    if mouse_event.left_button 
                    self.last_mouse_event = mouse_event;
                }
                _ => (),
            }
        }
    }
}

pub fn main() {
    match args().get(1) {
        Option::Some(arg) => FileManager::new().main(arg),
        Option::None => FileManager::new().main("file:///"),
    }
}
