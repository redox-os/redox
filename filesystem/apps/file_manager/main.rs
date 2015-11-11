use redox::Box;
use redox::{cmp, env};
use redox::collections::BTreeMap;
use redox::fs::{self, File};
use redox::io::{Read, Seek, SeekFrom};
use redox::time::{self, Duration};
use redox::vec::Vec;
use redox::string::{String, ToString};

use orbital::{event, BmpFile, Color, EventOption, MouseEvent, Window};

struct FileType {
    description: &'static str,
    icon: BmpFile,
}


impl FileType {
    fn new(desc: &'static str, icon: &str) -> FileType {
        FileType { description: desc, icon: load_icon(icon) }
    }

}

struct FileTypesInfo {
    file_types: BTreeMap<&'static str, FileType>,
}

impl FileTypesInfo {
    pub fn new () -> FileTypesInfo {
        let mut file_types = BTreeMap::<&'static str, FileType>::new();
        file_types.insert("/",
                          FileType::new("Folder", "inode-directory"));
        file_types.insert("wav",
                          FileType::new("WAV audio", "audio-x-wav"));
        file_types.insert("bin",
                          FileType::new("Executable", "application-x-executable"));
        file_types.insert("bmp",
                          FileType::new("Bitmap Image", "image-x-generic"));
        file_types.insert("rs",
                          FileType::new("Rust source code", "text-x-makefile"));
        file_types.insert("crate",
                          FileType::new("Rust crate", "application-x-archive"));
        file_types.insert("rlib",
                          FileType::new("Static Rust library", "application-x-object"));
        file_types.insert("asm",
                          FileType::new("Assembly source", "text-x-makefile"));
        file_types.insert("list",
                          FileType::new("Disassembly source", "text-x-makefile"));
        file_types.insert("c",
                          FileType::new("C source code", "text-x-csrc"));
        file_types.insert("cpp",
                          FileType::new("C++ source code", "text-x-c++src"));
        file_types.insert("h",
                          FileType::new("C header", "text-x-chdr"));
        file_types.insert("sh",
                          FileType::new("Shell script", "text-x-script"));
        file_types.insert("lua",
                          FileType::new("Lua script", "text-x-script"));
        file_types.insert("txt",
                          FileType::new("Plain text document", "text-x-generic"));
        file_types.insert("md",
                          FileType::new("Markdown document", "text-x-generic"));
        file_types.insert("toml",
                          FileType::new("TOML document", "text-x-generic"));
        file_types.insert("json",
                          FileType::new("JSON document", "text-x-generic"));
        file_types.insert("REDOX",
                          FileType::new("Redox package", "text-x-generic"));
        file_types.insert("",
                          FileType::new("Unknown file", "unknown"));
        FileTypesInfo { file_types: file_types }
    }

    pub fn description_for(&self, file_name: &str) -> String {
        if file_name.ends_with('/') {
            self.file_types["/"].description.to_string()
        } else {
            let pos = file_name.rfind('.').unwrap_or(0) + 1;
            let ext = &file_name[pos..];
            if self.file_types.contains_key(ext) {
                self.file_types[ext].description.to_string()
            } else {
                self.file_types[""].description.to_string()
            }
        }
    }

    pub fn icon_for(&self, file_name: &str) -> &BmpFile {
        if file_name.ends_with('/') {
            &self.file_types["/"].icon
        } else {
            let pos = file_name.rfind('.').unwrap_or(0) + 1;
            let ext = &file_name[pos..];
            if self.file_types.contains_key(ext) {
                &self.file_types[ext].icon
            } else {
                &self.file_types[""].icon
            }
        }
    }
}

enum FileManagerCommand {
    ChangeDir(String),
    Execute(String),
    Redraw,
    Quit,
}

pub struct FileManager {
    file_types_info: FileTypesInfo,
    files: Vec<String>,
    file_sizes: Vec<String>,
    selected: isize,
    last_mouse_event: MouseEvent,
    click_time: Duration,
    window: Box<Window>,
}

fn load_icon(path: &str) -> BmpFile {
    let mut vec: Vec<u8> = Vec::new();
    if let Some(mut file) = File::open(&("file:///ui/mimetypes/".to_string() + path + ".bmp")) {
        file.read_to_end(&mut vec);
    }
    BmpFile::from_data(&vec)
}

impl FileManager {
    pub fn new() -> Self {
        FileManager {
            file_types_info: FileTypesInfo::new(),
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
            window: Window::new(-1,-1,0,0,"").unwrap(),
        }
    }

    fn draw_content(&mut self) {
        self.window.set(Color::WHITE);

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
                let width = self.window.width();
                self.window.rect(0, 32 * row as isize, width, 32, Color::rgba(224, 224, 224, 255));
            }

            let icon = self.file_types_info.icon_for(&file_name);
            self.window.image(0,
                              32 * row as isize,
                              icon.width(),
                              icon.height(),
                              icon.as_slice());

            let mut col = 0;
            for c in file_name.chars() {
                if c == '\n' {
                    col = 0;
                    row += 1;
                } else if c == '\t' {
                    col += 8 - col % 8;
                } else {
                    if col < self.window.width() / 8 && row < self.window.height() / 32 {
                        self.window.char(8 * col as isize + 40,
                                    32 * row as isize + 8,
                                    c,
                                    Color::BLACK);
                        col += 1;
                    }
                }
                if col >= self.window.width() / 8 {
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
                    if col < self.window.width() / 8 && row < self.window.height() / 32 {
                        self.window.char(8 * col as isize + 40,
                                    32 * row as isize + 8,
                                    c,
                                    Color::BLACK);
                        col += 1;
                    }
                }
                if col >= self.window.width() / 8 {
                    col = 0;
                    row += 1;
                }
            }

            col = column[1];

            let description = self.file_types_info.description_for(&file_name);
            for c in description.chars() {
                if c == '\n' {
                    col = 0;
                    row += 1;
                } else if c == '\t' {
                    col += 8 - col % 8;
                } else {
                    if col < self.window.width() / 8 && row < self.window.height() / 32 {
                        self.window.char(8 * col as isize + 40,
                                    32 * row as isize + 8,
                                    c,
                                    Color::BLACK);
                        col += 1;
                    }
                }
                if col >= self.window.width() / 8 {
                    col = 0;
                    row += 1;
                }
            }

            row += 1;
            i += 1;
        }

        self.window.sync();
    }

    fn set_path(&mut self, path: &str) {
        let mut width = [48, 48, 48];
        let mut height = 0;
        if let Some(readdir) = fs::read_dir(path) {
            self.files.clear();
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
                let description = self.file_types_info.description_for(entry.path());
                width[0] = cmp::max(width[0], 48 + (entry.path().len()) * 8);
                width[1] = cmp::max(width[1], 8 + (self.file_sizes.last().unwrap().len()) * 8);
                width[2] = cmp::max(width[2], 8 + (description.len()) * 8);
            }

            if height < self.files.len() * 32 {
                height = self.files.len() * 32;
            }
        }
        // TODO: HACK ALERT - should use resize whenver that gets added
        self.window.sync_path();
        self.window = Window::new(self.window.x(),
                                  self.window.y(),
                                  width.iter().sum(),
                                  height,
                                  &path).unwrap();
        self.draw_content();
    }

    fn event_loop(&mut self) -> Option<FileManagerCommand> {
        let mut redraw = false;
        let mut command = None;
        if let Some(event) = self.window.poll() {
            match event.to_option() {
                EventOption::Key(key_event) => {
                    if key_event.pressed {
                        match key_event.scancode {
                            event::K_ESC => return Some(FileManagerCommand::Quit),
                            event::K_HOME => self.selected = 0,
                            event::K_UP => if self.selected > 0 {
                                self.selected -= 1;
                                redraw = true;
                            },
                            event::K_END => self.selected = self.files.len() as isize - 1,
                            event::K_DOWN => if self.selected < self.files.len() as isize - 1 {
                                self.selected += 1;
                                redraw = true;
                            },
                            _ => match key_event.character {
                                '\0' => (),
                                '\n' => {
                                    if self.selected >= 0 &&
                                       self.selected < self.files.len() as isize {
                                        match self.files.get(self.selected as usize) {
                                            Some(file) => {
                                                if file.ends_with('/') {
                                                    command = Some(FileManagerCommand::ChangeDir(file.clone()));
                                                } else {
                                                    command = Some(FileManagerCommand::Execute(file.clone()));
                                                }
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
                        if command.is_none() && redraw {
                            command = Some(FileManagerCommand::Redraw);
                        }
                    }
                }
                EventOption::Mouse(mouse_event) => {
                    redraw = true;
                    let mut i = 0;
                    let mut row = 0;
                    for file in self.files.iter() {
                        let mut col = 0;
                        for c in file.chars() {
                            if mouse_event.y >= 32 * row as isize &&
                               mouse_event.y < 32 * row as isize + 32 {
                                self.selected = i;
                            }

                            if c == '\n' {
                                col = 0;
                                row += 1;
                            } else if c == '\t' {
                                col += 8 - col % 8;
                            } else {
                                if col < self.window.width() / 8 && row < self.window.height() / 32 {
                                    col += 1;
                                }
                            }
                            if col >= self.window.width() / 8 {
                                col = 0;
                                row += 1;
                            }
                        }
                        row += 1;
                        i += 1; }

                    //Check for double click
                    if mouse_event.left_button {
                        let click_time = Duration::realtime();

                        if click_time - self.click_time < Duration::new(0, 500 * time::NANOS_PER_MILLI)
                            && self.last_mouse_event.x == mouse_event.x
                            && self.last_mouse_event.y == mouse_event.y {
                            if self.selected >= 0 && self.selected < self.files.len() as isize {
                                if let Some(file) = self.files.get(self.selected as usize) {
                                    if file.ends_with('/') {
                                        command = Some(FileManagerCommand::ChangeDir(file.clone()));
                                    } else {
                                        command = Some(FileManagerCommand::Execute(file.clone()));
                                    }
                                }
                            }
                            self.click_time = Duration::new(0, 0);
                        } else {
                            self.click_time = click_time;
                        }
                    }
                    self.last_mouse_event = mouse_event;

                    if command.is_none() && redraw {
                        command = Some(FileManagerCommand::Redraw);
                    }
                }
                EventOption::Quit(quit_event) => command = Some(FileManagerCommand::Quit),
                _ => (),
            }
        }
        command
    }

    fn main(&mut self, path: &str) {
        let mut current_path = path.to_string();
        self.set_path(path);
        loop {
            if let Some(event) = self.event_loop() {
                match event {
                    FileManagerCommand::ChangeDir(dir) => {
                        current_path = current_path + &dir;
                        self.set_path(&current_path);
                    },
                    FileManagerCommand::Execute(cmd) => {
                        //TODO: What is the best way to request a launch?
                        File::open(&("orbital://launch/".to_string() + &current_path + &cmd));
                    } ,
                    FileManagerCommand::Redraw => (),
                    FileManagerCommand::Quit => break,
                };
                self.draw_content();
            }
        }

    }
}

pub fn main() {
    match env::args().get(1) {
        Some(arg) => FileManager::new().main(arg),
        None => FileManager::new().main("file:/"),
    }
}
