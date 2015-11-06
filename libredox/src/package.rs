use alloc::boxed::Box;
use fs::File;
use io::*;
use string::*;
use vec::Vec;

use graphics::bmp::BmpFile;

use Url;

pub struct Package {
    pub path: String,
    pub id: String,
    pub name: String,
    pub binary: Url,
    pub icon: BmpFile,
    pub accepts: Vec<String>,
    pub authors: Vec<String>,
    pub descriptions: Vec<String>,
}

impl Package {
    pub fn from_path(path: &str) -> Box<Self> {
        let mut package = box Package {
            path: path.to_string(),
            id: String::new(),
            name: String::new(),
            binary: Url::new(),
            icon: BmpFile::new(0,0),
            accepts: Vec::new(),
            authors: Vec::new(),
            descriptions: Vec::new(),
        };

        let path_parts = Url::from_string(package.path.clone()).path_parts();
        if path_parts.len() > 0 {
            if let Some(part) = path_parts.get(path_parts.len() - 2).clone() {
                package.id = part.clone();
                package.binary = Url::from_string(path.to_string() + part + ".bin");
            }
        }

        let mut info = String::new();

        if let Some(mut resource) = File::open(&(path.to_string() + "_REDOX")) {
            resource.read_to_end(unsafe { info.as_mut_vec() });
        }

        for line in info.lines_any() {
            if line.starts_with("name=") {
                package.name = line[5 ..].to_string();
            } else if line.starts_with("binary=") {
                package.binary = Url::from_string(path.to_string() + &line[7 ..]);
            } else if line.starts_with("icon=") {
                if let Some(mut resource) = File::open(&line[5 ..].to_string()) {
                    // TODO: refactor this to just load from file
                    // LazyOxen
                    let mut vec: Vec<u8> = Vec::new();
                    resource.read_to_end(&mut vec);
                    package.icon = BmpFile::from_data(&vec);
                }
            } else if line.starts_with("accept=") {
                package.accepts.push(line[7 ..].to_string());
            } else if line.starts_with("author=") {
                package.authors.push(line[7 ..].to_string());
            } else if line.starts_with("description=") {
                package.descriptions.push(line[12 ..].to_string());
            } else {
                //TODO: print some kind of diagnostic?
                // ignore...
            }
        }

        package
    }
}
