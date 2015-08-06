use core::clone::Clone;
use core::ops::Fn;
use core::option::Option;

use alloc::boxed::*;

use common::memory::*;
use common::safeptr::*;
use common::string::*;
use common::url::*;

use filesystems::unfs::*;

use programs::session::*;

pub struct FileScheme;

impl SessionModule for FileScheme {
    fn scheme(&self) -> String {
        return "file".to_string();
    }

    #[allow(unused_variables)]
    fn on_url(&mut self, session: &Session, url: &URL) -> String{
        let mut ret = String::new();

        let unfs = UnFS::new();

        let mut path = String::new();
        for part in url.path.iter(){
            if path.len() > 0 {
                path = path + "/" + part.clone();
            }else{
                path = part.clone();
            }
        }

        for file in unfs.list(path.clone()).iter() {
            if ret.len() > 0 {
                ret = ret + "\n" + file.clone();
            }else{
                ret = file.clone();
            }
        }

        return ret;
    }

    #[allow(unused_variables)]
    fn on_url_async(&mut self, session: &Session, url: &URL, callback: Box<Fn(String)>){
        unsafe{
            let unfs = UnFS::new();

            let mut path = String::new();
            for part in url.path.iter(){
                if path.len() > 0 {
                    path = path + "/" + part.clone();
                }else{
                    path = part.clone();
                }
            }

            let node = unfs.node(path);

            if node as usize > 0{
                if (*node).data_sector_list.address > 0 {
                    let sector_list_ptr: SafePtr<SectorList> = SafePtr::new();
                    match sector_list_ptr.get() {
                        Option::Some(sector_list) => {
                            unfs.disk.read((*node).data_sector_list.address, 1, sector_list_ptr.unsafe_ptr() as usize);

                            for i in 0..1 {
                                if sector_list.extents[i].block.address > 0 && sector_list.extents[i].length > 0{
                                    session.on_url_async(&URL::from_string("ide:///".to_string() + sector_list.extents[i].block.address as usize + "/" + sector_list.extents[i].length as usize), callback);
                                    break;
                                }
                            }
                        },
                        Option::None => ()
                    }
                }

                unalloc(node as usize);
            }
        }
    }
}
