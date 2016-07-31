#![deny(warnings)]

extern crate orbclient;
extern crate orbimage;
extern crate orbfont;

use std::{cmp, env};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::process::Command;
use std::string::{String, ToString};
use std::vec::Vec;

use orbclient::{event, Color, EventOption, MouseEvent, Window};
use orbimage::Image;
use orbfont::Font;

struct FileType {
    description: &'static str,
    icon: Image,
}


impl FileType {
    fn new(desc: &'static str, icon: &str) -> FileType {
        FileType {
            description: desc,
            icon: load_icon(icon),
        }
    }
}

struct FileTypesInfo {
    file_types: BTreeMap<&'static str, FileType>,
}

impl FileTypesInfo {
    pub fn new() -> FileTypesInfo {
        let mut file_types = BTreeMap::<&'static str, FileType>::new();
        file_types.insert("/", FileType::new("Folder", "inode-directory"));
        file_types.insert("wav", FileType::new("WAV audio", "audio-x-wav"));
        file_types.insert("bin",
                          FileType::new("Executable", "application-x-executable"));
        file_types.insert("bmp", FileType::new("Bitmap Image", "image-x-generic"));
        file_types.insert("png", FileType::new("PNG Image", "image-x-generic"));
        file_types.insert("rs", FileType::new("Rust source code", "text-x-makefile"));
        file_types.insert("crate",
                          FileType::new("Rust crate", "application-x-archive"));
        file_types.insert("rlib",
                          FileType::new("Static Rust library", "application-x-object"));
        file_types.insert("asm", FileType::new("Assembly source", "text-x-makefile"));
        file_types.insert("list",
                          FileType::new("Disassembly source", "text-x-makefile"));
        file_types.insert("c", FileType::new("C source code", "text-x-csrc"));
        file_types.insert("cpp", FileType::new("C++ source code", "text-x-c++src"));
        file_types.insert("h", FileType::new("C header", "text-x-chdr"));
        file_types.insert("ion", FileType::new("Ion script", "text-x-script"));
        file_types.insert("rc", FileType::new("Init script", "text-x-script"));
        file_types.insert("sh", FileType::new("Shell script", "text-x-script"));
        file_types.insert("lua", FileType::new("Lua script", "text-x-script"));
        file_types.insert("conf", FileType::new("Config file", "text-x-generic"));
        file_types.insert("txt", FileType::new("Plain text file", "text-x-generic"));
        file_types.insert("md", FileType::new("Markdown file", "text-x-generic"));
        file_types.insert("toml", FileType::new("TOML file", "text-x-generic"));
        file_types.insert("json", FileType::new("JSON file", "text-x-generic"));
        file_types.insert("REDOX", FileType::new("Redox package", "text-x-generic"));
        file_types.insert("", FileType::new("Unknown file", "unknown"));
        FileTypesInfo { file_types: file_types }
    }

    pub fn description_for(&self, file_name: &str) -> String {
        if file_name.ends_with('/') {
            self.file_types["/"].description.to_owned()
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

    pub fn icon_for(&self, file_name: &str) -> &Image {
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
    files: Vec<(String, String)>,
    selected: isize,
    last_mouse_event: MouseEvent,
    window: Window,
    font: Font,
}

fn load_icon(path: &str) -> Image {
    match Image::from_path(&format!("/ui/mimetypes/{}.png", path)) {
        Ok(icon) => icon,
        Err(err) => {
            println!("Failed to load icon {}: {}", path, err);
            Image::new(32, 32)
        }
    }
}

impl FileManager {
    pub fn new() -> Self {
        FileManager {
            file_types_info: FileTypesInfo::new(),
            files: Vec::new(),
            selected: -1,
            last_mouse_event: MouseEvent {
                x: 0,
                y: 0,
                left_button: false,
                middle_button: false,
                right_button: false,
            },
            window: Window::new(-1, -1, 0, 0, "").unwrap(),
            font: Font::find(None, None, None).unwrap()
        }
    }

    fn draw_content(&mut self) {
        self.window.set(Color::rgb(255, 255, 255));

        let mut i = 0;
        let mut row = 0;
        let column = {
            let mut tmp = [0; 2];
            for file in self.files.iter() {
                if tmp[0] < file.0.len() {
                    tmp[0] = file.0.len();
                }
                if tmp[1] < file.1.len() {
                    tmp[1] = file.1.len();
                }
            }

            tmp[0] += 1;

            tmp[1] += tmp[0] + 1;

            tmp
        };
        for file in self.files.iter() {
            if i == self.selected {
                let width = self.window.width();
                self.window.rect(0,
                                 32 * row as i32,
                                 width,
                                 32,
                                 Color::rgba(224, 224, 224, 255));
            }

            let icon = self.file_types_info.icon_for(&file.0);
            icon.draw(&mut self.window, 0, 32 * row as i32);

            let mut col = 0;
            self.font.render(&file.0, 16.0).draw(&mut self.window, 8 * col as i32 + 40, 32 * row as i32 + 8, Color::rgb(0, 0, 0));

            col = column[0] as u32;
            self.font.render(&file.1, 16.0).draw(&mut self.window, 8 * col as i32 + 40, 32 * row as i32 + 8, Color::rgb(0, 0, 0));

            col = column[1] as u32;
            let description = self.file_types_info.description_for(&file.0);
            self.font.render(&description, 16.0).draw(&mut self.window, 8 * col as i32 + 40, 32 * row as i32 + 8, Color::rgb(0, 0, 0));

            row += 1;
            i += 1;
        }

        self.window.sync();
    }

    fn get_parent_directory() -> Option<String> {
        match File::open("../") {
            Ok(parent_dir) => match parent_dir.path() {
                Ok(path) => return Some(path.into_os_string().into_string().unwrap_or("/".to_string())),
                Err(err) => println!("failed to get path: {}", err)
            },
            Err(err) => println!("failed to open parent dir: {}", err)
        }

        None
    }

    fn get_num_entries(path: &str) -> String {
        let count = match fs::read_dir(path) {
            Ok(entry_readdir) => entry_readdir.count(),
            Err(_) => 0,
        };
        if count == 1 {
            "1 entry".to_string()
        } else {
            format!("{} entries", count)
        }
    }

    fn set_path(&mut self, path: &str) {
        let mut width = [48; 3];
        let mut height = 0;

        if let Err(err) = env::set_current_dir(path) {
            println!("failed to set dir {}: {}", path, err);
        }

        match fs::read_dir(path) {
            Ok(readdir) => {
                self.files.clear();

                // check to see if parent directory exists
                if let Some(parent_dir) = FileManager::get_parent_directory() {
                    self.files.push(("../".to_string(), FileManager::get_num_entries(&parent_dir)));
                }

                for entry_result in readdir {
                    match entry_result {
                        Ok(entry) => {
                            let directory = match entry.file_type() {
                                Ok(file_type) => file_type.is_dir(),
                                Err(err) => {
                                    println!("Failed to read file type: {}", err);
                                    false
                                }
                            };

                            let entry_path = match entry.file_name().to_str() {
                                Some(path_str) => if directory {
                                    path_str.to_string() + "/"
                                } else {
                                    path_str.to_string()
                                },
                                None => {
                                    println!("Failed to read file name");
                                    String::new()
                                }
                            };

                            self.files.push((entry_path.clone(), if directory {
                                FileManager::get_num_entries(&(path.to_string() + &entry_path))
                            } else {
                                match fs::metadata(&entry_path) {
                                    Ok(metadata) => {
                                        let size = metadata.len();
                                        if size >= 1_000_000_000 {
                                            format!("{:.1} GB", (size as u64) / 1_000_000_000)
                                        } else if size >= 1_000_000 {
                                            format!("{:.1} MB", (size as u64) / 1_000_000)
                                        } else if size >= 1_000 {
                                            format!("{:.1} KB", (size as u64) / 1_000)
                                        } else {
                                            format!("{:.1} bytes", size)
                                        }
                                    }
                                    Err(err) => format!("Failed to open: {}", err),
                                }
                            }));
                            // Unwrapping the last file size will not panic since it has
                            // been at least pushed once in the vector
                            let description = self.file_types_info.description_for(&entry_path);
                            width[0] = cmp::max(width[0], 48 + (entry_path.len()) * 8);
                            width[1] = cmp::max(width[1], 8 + (self.files.last().unwrap().1.len()) * 8);
                            width[2] = cmp::max(width[2], 8 + (description.len()) * 8);
                        },
                        Err(err) => println!("failed to read dir entry: {}", err)
                    }
                }

                self.files.sort_by(|a, b| {
                    a.0.cmp(&b.0)
                });

                if height < self.files.len() * 32 {
                    height = self.files.len() * 32;
                }
            },
            Err(err) => println!("failed to readdir {}: {}", path, err)
        }

        // TODO: HACK ALERT - should use resize whenver that gets added
        self.window.sync_path();

        let x = self.window.x();
        let y = self.window.y();
        let w = width.iter().sum::<usize>() as u32;
        let h = height as u32;

        println!("new window: {},{} {}x{}", x, y, w, h);
        self.window = Window::new(x, y, w, h, &path).unwrap();

        self.draw_content();
    }

    fn event_loop(&mut self) -> Vec<FileManagerCommand> {
        let mut redraw = false;
        let mut commands = Vec::new();
        for event in self.window.events() {
            match event.to_option() {
                EventOption::Key(key_event) => {
                    if key_event.pressed {
                        match key_event.scancode {
                            event::K_ESC => commands.push(FileManagerCommand::Quit),
                            event::K_HOME => self.selected = 0,
                            event::K_UP => {
                                if self.selected > 0 {
                                    self.selected -= 1;
                                    redraw = true;
                                }
                            },
                            event::K_END => self.selected = self.files.len() as isize - 1,
                            event::K_DOWN => {
                                if self.selected < self.files.len() as isize - 1 {
                                    self.selected += 1;
                                    redraw = true;
                                }
                            },
                            _ => {
                                match key_event.character {
                                    '\0' => (),
                                    '\n' => {
                                        if self.selected >= 0 &&
                                           self.selected < self.files.len() as isize {
                                            match self.files.get(self.selected as usize) {
                                                Some(file) => {
                                                    if file.0.ends_with('/') {
                                                        commands.push(FileManagerCommand::ChangeDir(file.0.clone()));
                                                    } else {
                                                        commands.push(FileManagerCommand::Execute(file.0.clone()));
                                                    }
                                                }
                                                None => (),
                                            }
                                        }
                                    }
                                    _ => {
                                        let mut i = 0;
                                        for file in self.files.iter() {
                                            if file.0.starts_with(key_event.character) {
                                                self.selected = i;
                                                break;
                                            }
                                            i += 1;
                                        }
                                    }
                                }
                            }
                        }
                        if redraw {
                            commands.push(FileManagerCommand::Redraw);
                        }
                    }
                }
                EventOption::Mouse(mouse_event) => {
                    redraw = true;
                    let mut i = 0;
                    let mut row = 0;
                    for file in self.files.iter() {
                        let mut col = 0;
                        for c in file.0.chars() {
                            if mouse_event.y >= 32 * row as i32 &&
                               mouse_event.y < 32 * row as i32 + 32 {
                                self.selected = i;
                            }

                            if c == '\n' {
                                col = 0;
                                row += 1;
                            } else if c == '\t' {
                                col += 8 - col % 8;
                            } else {
                                if col < self.window.width() / 8 &&
                                   row < self.window.height() / 32 {
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

                    if mouse_event.left_button {
                        if self.last_mouse_event.x == mouse_event.x &&
                           self.last_mouse_event.y == mouse_event.y {
                            if self.selected >= 0 && self.selected < self.files.len() as isize {
                                if let Some(file) = self.files.get(self.selected as usize) {
                                    if file.0.ends_with('/') {
                                        commands.push(FileManagerCommand::ChangeDir(file.0.clone()));
                                    } else {
                                        commands.push(FileManagerCommand::Execute(file.0.clone()));
                                    }
                                }
                            }
                        }
                    }
                    self.last_mouse_event = mouse_event;

                    if redraw {
                        commands.push(FileManagerCommand::Redraw);
                    }
                }
                EventOption::Quit(_) => commands.push(FileManagerCommand::Quit),
                _ => (),
            }
        }
        commands
    }

    fn main(&mut self, path: &str) {
        let mut current_path = path.to_string();
        self.set_path(path);
        self.draw_content();
        'events: loop {
            let mut redraw = false;
            for event in self.event_loop() {
                match event {
                    FileManagerCommand::ChangeDir(dir) => {
                        if dir == "../" {
                            println!("Change dir up");
                            if let Some(parent_dir) = FileManager::get_parent_directory() {
                                current_path = parent_dir;
                            }
                        } else {
                            current_path = current_path + &dir;
                        }
                        self.set_path(&current_path);
                    }
                    FileManagerCommand::Execute(cmd) => {
                        Command::new("launcher").arg(&(current_path.clone() + &cmd)).spawn().unwrap();
                    },
                    FileManagerCommand::Redraw => redraw = true,
                    FileManagerCommand::Quit => break 'events,
                };
            }
            if redraw {
                self.draw_content();
            }
        }
        println!("Exited");
    }
}

fn main() {
    match env::args().nth(1) {
        Some(ref arg) => FileManager::new().main(arg),
        None => FileManager::new().main("/home/"),
    }
}
