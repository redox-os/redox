use core::clone::Clone;

use common::string::*;
use common::url::*;

use filesystems::unfs::*;

use programs::session::*;

pub struct FileScheme;

impl SessionScheme for FileScheme {
    fn scheme(&self) -> String {
        return "file".to_string();
    }

    #[allow(unused_variables)]
    fn on_url(&mut self, session: &Session, url: &URL) -> String {
        let unfs = UnFS::new();

        let mut path = String::new();
        for part in url.path.as_slice(){
            if path.len() > 0 {
                path = path + "/" + part.clone();
            }else{
                path = part.clone();
            }
        }

        let mut ret = String::new();
        for file in unfs.list(path.clone()).as_slice() {
            if ret.len() > 0 {
                ret = ret + "\n" + file.clone();
            }else{
                ret = file.clone();
            }
        }
        return ret;
    }
}
