use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;

use common::debug;

use graphics::bmp::BMPFile;

use schemes::URL;

pub struct Package {
    pub url: URL,
    pub id: String,
    pub name: String,
    pub binary: URL,
    pub icon: BMPFile,
    pub accepts: Vec<String>,
    pub authors: Vec<String>,
    pub descriptions: Vec<String>,
}

impl Package {
    pub fn from_url(url: &URL) -> Box<Self> {
        let mut package = box Package {
            url: url.clone(),
            id: String::new(),
            name: String::new(),
            binary: URL::new(),
            icon: BMPFile::new(),
            accepts: Vec::new(),
            authors: Vec::new(),
            descriptions: Vec::new(),
        };

        let path_parts = url.path_parts();
        if path_parts.len() > 0 {
            if let Some(part) = path_parts.get(path_parts.len() - 1) {
                package.id = part.clone();
                package.binary = URL::from_string(&(url.to_string() + part + ".bin"));
            }
        }

        let mut info = String::new();
        if let Some(mut resource) = URL::from_string(&(url.to_string() + "_REDOX")).open() {
            let mut vec: Vec<u8> = Vec::new();
            resource.read_to_end(&mut vec);
            info = String::from_utf8_unchecked(vec);
        }

        for line in info.lines() {
            if line.starts_with("name=".to_string()) {
                package.name = line[5 ..].to_string();
            } else if line.starts_with("binary=".to_string()) {
                package.binary = URL::from_string(&(url.to_string() + &line[7 ..]));
            } else if line.starts_with("icon=".to_string()) {
                if let Some(mut resource) = URL::from_str(&line[5 ..]).open() {
                    let mut vec: Vec<u8> = Vec::new();
                    resource.read_to_end(&mut vec);
                    package.icon = BMPFile::from_data(&vec);
                }
            } else if line.starts_with("accept=".to_string()) {
                package.accepts.push(line[7 ..].to_string());
            } else if line.starts_with("author=".to_string()) {
                package.authors.push(line[7 ..].to_string());
            } else if line.starts_with("description=".to_string()) {
                package.descriptions.push(line[12 ..].to_string());
            } else {
                debug::d("Unknown package info: ");
                debug::d(&line);
                debug::dl();
            }
        }

        package
    }
}
