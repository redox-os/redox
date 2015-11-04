use ::GetSlice;

use alloc::boxed::Box;

use collections::string::{String, ToString};
use collections::vec::Vec;

use common::debug;
use common::parse_path::parse_path;

use graphics::bmp::BmpFile;

use schemes::Url;

/// A package (_REDOX content serialized)
pub struct Package {
    /// The URL
    pub url: Url,
    /// The ID of the package
    pub id: String,
    /// The name of the package
    pub name: String,
    /// The binary for the package
    pub binary: Url,
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
    pub fn from_url(url: &Url) -> Box<Self> {
        let mut package = box Package {
            url: url.clone(),
            id: String::new(),
            name: String::new(),
            binary: Url::new(),
            icon: BmpFile::new(),
            accepts: Vec::new(),
            authors: Vec::new(),
            descriptions: Vec::new(),
        };

        let path_parts = parse_path(url.reference());

        if !path_parts.is_empty() {
            if let Some(part) = path_parts.get(path_parts.len() - 1) {
                package.id = part.clone();
                package.binary = Url::from_string(&(url.to_string() + part + ".bin"));
            }
        }

        let mut info = String::new();

        if let Some(mut resource) = Url::from_string(&(url.to_string() + "_REDOX")).open() {
            resource.read_to_end(unsafe { info.as_mut_vec() });
        }

        for line in info.lines_any() {
            if line.starts_with("name=") {
                package.name = line.get_slice(Some(5), None).to_string();
            } else if line.starts_with("binary=") {
                package.binary = Url::from_string(&(url.to_string() + line.get_slice(Some(7), None)));
            } else if line.starts_with("icon=") {
                if let Some(mut resource) = Url::from_string(&line.get_slice(Some(5), None).to_string()).open() {
                    let mut vec: Vec<u8> = Vec::new();
                    resource.read_to_end(&mut vec);
                    package.icon = BmpFile::from_data(&vec);
                }
            } else if line.starts_with("accept=") {
                package.accepts.push(line.get_slice(Some(7), None).to_string());
            } else if line.starts_with("author=") {
                package.authors.push(line.get_slice(Some(7), None).to_string());
            } else if line.starts_with("description=") {
                package.descriptions.push(line.get_slice(Some(12), None).to_string());
            } else {
                debug::d("Unknown package info: ");
                debug::d(&line);
                debug::dl();
            }
        }

        package
    }
}
