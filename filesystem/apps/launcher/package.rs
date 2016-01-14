use std::get_slice::GetSlice;
use std::fs::File;
use std::io::Read;

use orbital::BmpFile;

/// A package (_REDOX content serialized)
pub struct Package {
    /// The URL
    pub url: String,
    /// The ID of the package
    pub id: String,
    /// The name of the package
    pub name: String,
    /// The binary for the package
    pub binary: String,
    /// The icon for the package
    pub icon: BmpFile,
    /// The accepted extensions
    pub accepts: Vec<String>,
    /// The author(s) of the package
    pub authors: Vec<String>,
    /// The description of the package
    pub descriptions: Vec<String>,
}

impl Package {
    /// Create package from URL
    pub fn from_path(url: &str) -> Box<Self> {
        let mut package = Box::new(Package {
            url: url.to_string(),
            id: String::new(),
            name: String::new(),
            binary: url.to_string() + "main.bin",
            icon: BmpFile::default(),
            accepts: Vec::new(),
            authors: Vec::new(),
            descriptions: Vec::new(),
        });

        for part in url.rsplit('/') {
            if !part.is_empty() {
                package.id = part.to_string();
                break;
            }
        }

        let mut info = String::new();

        if let Ok(mut file) = File::open(&(url.to_string() + "_REDOX")) {
            file.read_to_string(&mut info);
        }

        for line in info.lines() {
            if line.starts_with("name=") {
                package.name = line.get_slice(Some(5), None).to_string();
            } else if line.starts_with("binary=") {
                package.binary = url.to_string() + line.get_slice(Some(7), None);
            } else if line.starts_with("icon=") {
                package.icon = BmpFile::from_path(line.get_slice(Some(5), None));
            } else if line.starts_with("accept=") {
                package.accepts.push(line.get_slice(Some(7), None).to_string());
            } else if line.starts_with("author=") {
                package.authors.push(line.get_slice(Some(7), None).to_string());
            } else if line.starts_with("description=") {
                package.descriptions.push(line.get_slice(Some(12), None).to_string());
            } else {
                println!("Unknown package info: {}", line);
            }
        }

        package
    }
}
