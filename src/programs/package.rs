use graphics::bmp::*;

use programs::common::*;

pub struct Package {
    pub url: URL,
    pub id: String,
    pub name: String,
    pub binary: URL,
    pub icon: BMP,
    pub accepts: Vec<String>,
    pub authors: Vec<String>,
    pub descriptions: Vec<String>
}

impl Package {
    pub fn from_url(url: &URL) -> Box<Package> {
        let mut package = box Package {
            url: url.clone(),
            id: String::new(),
            name: String::new(),
            binary: URL::new(),
            icon: BMP::new(),
            accepts: Vec::new(),
            authors: Vec::new(),
            descriptions: Vec::new()
        };

        let path_parts = url.path_parts();
        if path_parts.len() > 0 {
            if let Option::Some(part) = path_parts.get(path_parts.len() - 1) {
                package.id = part.clone();
                package.binary = URL::from_string(&(url.to_string() + part + ".bin"));
            }
        }

        let info;
        {
            let mut resource = URL::from_string(&(url.to_string() + "_REDOX")).open();
            let mut vec: Vec<u8> = Vec::new();
            resource.read_to_end(&mut vec);
            info = String::from_utf8(&vec);
        }

        for line in info.split("\n".to_string()) {
            if line.starts_with("name=".to_string()) {
                package.name = line.substr(5, line.len() - 5);
            } else if line.starts_with("binary=".to_string()) {
                package.binary = URL::from_string(&(url.to_string() + line.substr(7, line.len() - 7)));
            } else if line.starts_with("icon=".to_string()) {
                let mut resource = URL::from_string(&line.substr(5, line.len() - 5)).open();
                let mut vec: Vec<u8> = Vec::new();
                resource.read_to_end(&mut vec);
                package.icon = BMP::from_data(&vec);
            } else if line.starts_with("accept=".to_string()) {
                package.accepts.push(line.substr(7, line.len() - 7));
            } else if line.starts_with("author=".to_string()) {
                package.authors.push(line.substr(7, line.len() - 7));
            } else if line.starts_with("description=".to_string()) {
                package.descriptions.push(line.substr(12, line.len() - 12));
            } else {
                d("Unknown package info: ");
                line.d();
                dl();
            }
        }

        package
    }

    pub fn d(&self) {
        d("URL: ");
        self.url.d();
        dl();

        d("ID: ");
        self.id.d();
        dl();

        d("Name: ");
        self.name.d();
        dl();

        d("Binary: ");
        self.binary.d();
        dl();

        d("Icon: ");
        dd(self.icon.size.width);
        d("x");
        dd(self.icon.size.height);
        dl();

        for accept in self.accepts.iter() {
            d("Accept: ");
            accept.d();
            dl();
        }

        for author in self.authors.iter() {
            d("Author: ");
            author.d();
            dl();
        }

        for description in self.descriptions.iter() {
            d("Description: ");
            description.d();
            dl();
        }
    }
}
