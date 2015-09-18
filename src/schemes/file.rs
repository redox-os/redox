use filesystems::unfs::*;

use programs::common::*;

pub struct FileScheme{
    pub unfs: UnFS
}

impl SessionItem for FileScheme {
    fn scheme(&self) -> String {
        return "file".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource>{
        let path = url.path();
        match self.unfs.node(&path) {
            Option::Some(node) => {
                if node.extents[0].block > 0 && node.extents[0].length > 0{
                    return URL::from_string(&("ide:///".to_string() + node.extents[0].block as usize + "/" + node.extents[0].length as usize)).open();
                }else{
                    return box NoneResource;
                }
            },
            Option::None => {
                let mut list = String::new();
                let mut dirs: Vec<String> = Vec::new();

                for file in self.unfs.list(&path).iter() {
                    let line;
                    match file.find("/".to_string()) {
                        Option::Some(index) => {
                            let dirname = file.substr(0, index + 1);
                            let mut found = false;
                            for dir in dirs.iter() {
                                if dirname == *dir {
                                    found = true;
                                    break;
                                }
                            }
                            if found {
                                line = String::new();
                            }else{
                                line = dirname.clone();
                                dirs.push(dirname);
                            }
                        },
                        Option::None => line = file.clone()
                    }
                    if line.len() > 0 {
                        if list.len() > 0 {
                            list = list + '\n' + line;
                        }else{
                            list = line;
                        }
                    }
                }

                return box VecResource::new(url.clone(), ResourceType::Dir, list.to_utf8());
            }
        }
    }
}
