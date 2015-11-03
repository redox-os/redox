use redox::{self, cmp, env, BMPFile, Color};
use redox::collections::BTreeMap;
use redox::event::{self, EventOption, MouseEvent};
use redox::fs::{self, File};
use redox::io::{Read, Seek, SeekFrom};
use redox::orbital::window::Window;
use redox::time::{self, Duration};
use redox::vec::Vec;
use redox::string::{String, ToString};

pub struct FileType {
    description: String,
    icon: BMPFile,
}

impl FileType {
    pub fn new(desc: &str, icon: &str) -> FileType {
        FileType { description: desc.to_string(), icon: load_icon(icon) }
    }

}

pub struct FileManager {
    file_types: BTreeMap<String, FileType>,
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
            file_types: {
                let mut file_types = BTreeMap::<String, FileType>::new();
                file_types.insert("/".to_string(),
                                  FileType::new("Folder", "inode-directory"));
                file_types.insert("wav".to_string(),
                                  FileType::new("WAV audio", "audio-x-wav"));
                file_types.insert("bin".to_string(),
                                  FileType::new("Executable", "application-x-executable"));
                file_types.insert("bmp".to_string(),
                                  FileType::new("Bitmap Image", "image-x-generic"));
                file_types.insert("rs".to_string(),
                                  FileType::new("Rust source code", "text-x-makefile"));
                file_types.insert("crate".to_string(),
                                  FileType::new("Rust crate", "application-x-archive"));
                file_types.insert("rlib".to_string(),
                                  FileType::new("Static Rust library", "application-x-object"));
                file_types.insert("asm".to_string(),
                                  FileType::new("Assembly source", "text-x-makefile"));
                file_types.insert("list".to_string(),
                                  FileType::new("Disassembly source", "text-x-makefile"));
                file_types.insert("c".to_string(),
                                  FileType::new("C source code", "text-x-csrc"));
                file_types.insert("cpp".to_string(),
                                  FileType::new("C++ source code", "text-x-c++src"));
                file_types.insert("h".to_string(),
                                  FileType::new("C header", "text-x-chdr"));
                file_types.insert("sh".to_string(),
                                  FileType::new("Shell script", "text-x-script"));
                file_types.insert("lua".to_string(),
                                  FileType::new("Lua script", "text-x-script"));
                file_types.insert("txt".to_string(),
                                  FileType::new("Plain text document", "text-x-generic"));
                file_types.insert("md".to_string(),
                                  FileType::new("Markdown document", "text-x-generic"));
                file_types.insert("toml".to_string(),
                                  FileType::new("TOML document", "text-x-generic"));
                file_types.insert("json".to_string(),
                                  FileType::new("JSON document", "text-x-generic"));
                file_types.insert("REDOX".to_string(),
                                  FileType::new("Redox package", "text-x-generic"));
                file_types.insert(String::new(),
                                  FileType::new("Unknown file", "unknown"));
                file_types
            },
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

    fn load_icon_with(&self, file_name: &str, row: isize, window: &mut Window) {
        if file_name.ends_with('/') {
            window.image(0,
                         32 * row as isize,
                         self.file_types["/"].icon.width(),
                         self.file_types["/"].icon.height(),
                         self.file_types["/"].icon.as_slice());
        } else {
            let pos = file_name.rfind('.').unwrap_or(0) + 1;
            match self.file_types.get(&file_name[pos..]) {
                Some(file_type) => {
                    window.image(0,
                                 32 * row,
                                 file_type.icon.width(),
                                 file_type.icon.height(),
                                 file_type.icon.as_slice());
                }
                None => {
                    window.image(0,
                                 32 * row,
                                 self.file_types[""].icon.width(),
                                 self.file_types[""].icon.height(),
                                 self.file_types[""].icon.as_slice());
                }
            }
        }
    }

    fn get_description(&self, file_name: &str) -> String {
        if file_name.ends_with('/') {
            self.file_types["/"].description.clone()
        } else {
            let pos = file_name.rfind('.').unwrap_or(0) + 1;
            match self.file_types.get(&file_name[pos..]) {
                Some(file_type) => file_type.description.clone(),
                None => self.file_types[""].description.clone(),
            }
        }
    }

    fn draw_content(&mut self, window: &mut Window) {
        window.set(Color::WHITE);

        let mut i = 0;
        let mut row = 0;
        let column = {
            let mut tmp = [0, 0];
            for string in self.files.iter() {
                if tmp[0] < string.len() {
                    tmp[0] = string.len();
                }
            }

            tmp[0] += 1;

            for file_size in self.file_sizes.iter() {
                if tmp[1] < file_size.len() {
                    tmp[1] = file_size.len();
                }
            }

            tmp[1] += tmp[0] + 1;
            tmp
        };
        for (file_name, file_size) in self.files.iter().zip(self.file_sizes.iter()) {
            if i == self.selected {
                let width = window.width();
                window.rect(0, 32 * row as isize, width, 32, Color::rgba(224, 224, 224, 255));
            }

            self.load_icon_with(&file_name, row as isize, window);

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
                                    Color::BLACK);
                        col += 1;
                    }
                }
                if col >= window.width() / 8 {
                    col = 0;
                    row += 1;
                }
            }

            col = column[0];

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
                                    Color::BLACK);
                        col += 1;
                    }
                }
                if col >= window.width() / 8 {
                    col = 0;
                    row += 1;
                }
            }

            col = column[1];

            for c in self.get_description(file_name).chars() {
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
                                    Color::BLACK);
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
        let mut width = [48, 48, 48];
        let mut height = 0;
        if let Some(readdir) = fs::read_dir(path) {
            for entry in readdir {
                self.files.push(entry.path().to_string());
                self.file_sizes.push(
                    // When the entry is a folder
                    if entry.path().ends_with('/') {
                        let count = match fs::read_dir(&(path.to_string() + entry.path())) {
                            Some(entry_readdir) => entry_readdir.count(),
                            None => 0
                        };

                        if count == 1 {
                            "1 entry".to_string()
                        } else {
                            format!("{} entries", count)
                        }
                    } else {
                        match File::open(&(path.to_string() + entry.path())) {
                            Some(mut file) => match file.seek(SeekFrom::End(0)) {
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
                            },
                            None => "Failed to open".to_string()
                        }
                    }
                );
                // Unwrapping the last file size will not panic since it has
                // been at least pushed once in the vector
                width[0] = cmp::max(width[0], 48 + (entry.path().len()) * 8);
                width[1] = cmp::max(width[1], 8 + (self.file_sizes.last().unwrap().len()) * 8);
                width[2] = cmp::max(width[2], 8 + (self.get_description(entry.path()).len()) * 8);
            }

            if height < self.files.len() * 32 {
                height = self.files.len() * 32;
            }
        }

        let mut window = Window::new((redox::rand() % 400 + 50) as isize,
                                     (redox::rand() % 300 + 50) as isize,
                                     width.iter().sum(),
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
                                if let Some(file) = self.files.get(self.selected as usize) {
                                    File::exec(&(path.to_string() + &file));
                                }
                            }
                            self.click_time = Duration::new(0, 0);
                        } else {
                            self.click_time = click_time;
                        }
                    }
                    self.last_mouse_event = mouse_event;
                }
                EventOption::Quit(quit_event) => break,
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
