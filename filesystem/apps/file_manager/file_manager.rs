use redox::{self, env, BMPFile};
use redox::event::{self, EventOption, MouseEvent};
use redox::fs::file::File;
use redox::io::{Read, Seek, SeekFrom};
use redox::orbital::Window;
use redox::time::{self, Duration};
use redox::vec::Vec;
use redox::string::{String, ToString};

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
    file_sizes: Vec<String>,
    selected: isize,
    last_mouse_event: MouseEvent,
    click_time: Duration,
}

fn load_icon(path: &str) -> BMPFile {
    let mut vec: Vec<u8> = Vec::new();
    if let Some(mut file) = File::open(&("file:///ui/mimetypes/".to_string() + path + ".bmp")) {
        file.read_to_end(&mut vec);
    }
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
            file_sizes: Vec::new(),
            selected: -1,
            last_mouse_event: MouseEvent {
                x: 0,
                y: 0,
                left_button: false,
                middle_button: false,
                right_button: false,
            },
            click_time: Duration::new(0, 0),
        }
    }

    fn draw_content(&mut self, window: &mut Window) {
        window.set([255, 255, 255, 255]);

        let mut i = 0;
        let mut row = 0;
        let column1 = {
            let mut tmp = 0;
            for string in self.files.iter() {
                if tmp < string.len() {
                    tmp = string.len();
                }
            }
            tmp + 1
        };
        for (file_name, file_size) in self.files.iter().zip(self.file_sizes.iter()) {
            if i == self.selected {
                let width = window.width();
                window.rect(0, 32 * row as isize, width, 32, [224, 224, 224, 255]);
            }

            if file_name.ends_with('/') {
                window.image(0,
                             32 * row as isize,
                             self.folder_icon.width(),
                             self.folder_icon.height(),
                             self.folder_icon.as_slice());
            } else if file_name.ends_with(".wav") {
                window.image(0,
                             32 * row as isize,
                             self.audio_icon.width(),
                             self.audio_icon.height(),
                             self.audio_icon.as_slice());
            } else if file_name.ends_with(".bin") {
                window.image(0,
                             32 * row as isize,
                             self.bin_icon.width(),
                             self.bin_icon.height(),
                             self.bin_icon.as_slice());
            } else if file_name.ends_with(".bmp") {
                window.image(0,
                             32 * row as isize,
                             self.image_icon.width(),
                             self.image_icon.height(),
                             self.image_icon.as_slice());
            } else if file_name.ends_with(".rs") || file_name.ends_with(".asm") || file_name.ends_with(".list") {
                window.image(0,
                             32 * row as isize,
                             self.source_icon.width(),
                             self.source_icon.height(),
                             self.source_icon.as_slice());
            } else if file_name.ends_with(".sh") || file_name.ends_with(".lua") {
                window.image(0,
                             32 * row as isize,
                             self.script_icon.width(),
                             self.script_icon.height(),
                             self.script_icon.as_slice());
            } else if file_name.ends_with(".md") || file_name.ends_with(".txt") {
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
            for c in file_name.chars() {
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

            col = column1;

            for c in file_size.chars() {
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
        if let Some(mut file) = File::open(path) {
            let mut list = String::new();
            file.read_to_string(&mut list);

            for entry in list.split('\n') {
                self.files.push(entry.to_string());
                self.file_sizes.push(
                    match File::open(&(path.to_string() + entry)) {
                        Some(mut file) => {
                            // When the entry is a folder
                            if entry.ends_with('/') {
                                let mut string = String::new();
                                file.read_to_string(&mut string);

                                let count = string.split('\n').count();
                                if count == 1 {
                                    "1 entry".to_string()
                                } else {
                                    format!("{} entries", count)
                                }
                            } else {
                                match file.seek(SeekFrom::End(0)) {
                                    Some(size) => {
                                        if size >= 1_000_000_000 {
                                            format!("{:.1} GB", (size as f64)/1_000_000_000.0)
                                        } else if size >= 1_000_000 {
                                            format!("{:.1} MB", (size as f64)/1_000_000.0)
                                        } else if size >= 1_000 {
                                            format!("{:.1} KB", (size as f64)/1_000.0)
                                        } else {
                                            format!("{:.1} bytes", size)
                                        }
                                    }
                                    None => "Failed to seek".to_string()
                                }
                            }
                        },
                        None => "Failed to open".to_string(),
                    }
                );
                // Unwrapping the last file size will not panic since it has
                // been at least pushed once in the vector
                let current_width = (40 + (entry.len() + 1) * 8) +
                                    (8 + (self.file_sizes.last().unwrap().len() + 1) * 8);
                if width < current_width {
                    width = current_width;
                }
            }

            if height < self.files.len() * 32 {
                height = self.files.len() * 32;
            }
        }

        let mut window = Window::new((redox::rand() % 400 + 50) as isize,
                                     (redox::rand() % 300 + 50) as isize,
                                     width,
                                     height,
                                     &path).unwrap();

        self.draw_content(&mut window);

        while let Some(event) = window.poll() {
            match event.to_option() {
                EventOption::Key(key_event) => {
                    if key_event.pressed {
                        match key_event.scancode {
                            event::K_ESC => break,
                            event::K_HOME => self.selected = 0,
                            event::K_UP => if self.selected > 0 {
                                self.selected -= 1;
                            },
                            event::K_END => self.selected = self.files.len() as isize - 1,
                            event::K_DOWN => if self.selected < self.files.len() as isize - 1 {
                                self.selected += 1;
                            },
                            _ => match key_event.character {
                                '\0' => (),
                                '\n' => {
                                    if self.selected >= 0 &&
                                       self.selected < self.files.len() as isize {
                                        match self.files.get(self.selected as usize) {
                                            Some(file) => {
                                                File::exec(&(path.to_string() + &file));
                                            },
                                            None => (),
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
                                self.selected = i;
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

                    //Check for double click
                    if mouse_event.left_button {
                        let click_time = Duration::realtime();

                        if click_time - self.click_time < Duration::new(0, 500 * time::NANOS_PER_MILLI)
                            && self.last_mouse_event.x == mouse_event.x
                            && self.last_mouse_event.y == mouse_event.y {
                            if self.selected >= 0 && self.selected < self.files.len() as isize {
                                match self.files.get(self.selected as usize) {
                                    Some(file) => {
                                        File::exec(&(path.to_string() + &file));
                                    },
                                    None => (),
                                }
                            }
                            self.click_time = Duration::new(0, 0);
                        } else {
                            self.click_time = click_time;
                        }
                    }
                    self.last_mouse_event = mouse_event;
                }
                _ => (),
            }
        }
    }
}

pub fn main() {
    match env::args().get(1) {
        Some(arg) => FileManager::new().main(arg),
        None => FileManager::new().main("file:///"),
    }
}
