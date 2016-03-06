use std::fs::File;
use std::io::Read;

pub struct Config {
    pub background: String,
    pub cursor: String,
}

impl Config {
    pub fn from_path(path: &str) -> Config {
        let mut string = String::new();

        match File::open(path) {
            Ok(mut file) => match file.read_to_string(&mut string) {
                Ok(_) => (),
                Err(err) => println!("orbital: failed to read config '{}': {}", path, err)
            },
            Err(err) => println!("orbital: failed to open config '{}': {}", path, err)
        }

        Config::from_str(&string)
    }

    pub fn from_str(string: &str) -> Config {
        let mut config = Config {
            background: String::new(),
            cursor: String::new(),
        };

        for line_original in string.lines() {
            let line = line_original.trim();
            if line.starts_with("background=") {
                config.background = line[11..].to_string();
            }
            if line.starts_with("cursor=") {
                config.cursor = line[7..].to_string();
            }
        }

        config
    }
}
