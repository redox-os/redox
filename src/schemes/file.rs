use filesystems::unfs::*;

use programs::common::*;

pub struct FileScheme{
    pub unfs: UnFS
}

impl SessionModule for FileScheme {
    fn scheme(&self) -> String {
        return "file".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource>{
        let path = url.path_string();
        match self.unfs.node(path.clone()) {
            Option::Some(node) => {
                if node.extents[0].block > 0 && node.extents[0].length > 0{
                    return URL::from_string("ide:///".to_string() + node.extents[0].block as usize + "/" + node.extents[0].length as usize).open();
                }else{
                    return box NoneResource;
                }
            },
            Option::None => {
                let mut list = String::new();

                for file in self.unfs.list(path.clone()).iter() {
                    if list.len() > 0 {
                        list = list + "\n" + file.clone();
                    }else{
                        list = file.clone();
                    }
                }

                return box VecResource::new(ResourceType::Dir, list.to_utf8());
            }
        }
    }

    fn open_async(&mut self, url: &URL, callback: Box<FnBox(Box<Resource>)>){
        let path = url.path_string();
        match self.unfs.node(path.clone()) {
            Option::Some(node) => {
                if node.extents[0].block > 0 && node.extents[0].length > 0{
                    URL::from_string("ide:///".to_string() + node.extents[0].block as usize + "/" + node.extents[0].length as usize).open_async(callback);
                }else{
                    callback(box NoneResource);
                }
            },
            Option::None => {
                let mut list = String::new();

                for file in self.unfs.list(path.clone()).iter() {
                    if list.len() > 0 {
                        list = list + "\n" + file.clone();
                    }else{
                        list = file.clone();
                    }
                }

                callback(box VecResource::new(ResourceType::Dir, list.to_utf8()));
            }
        }
    }
}
