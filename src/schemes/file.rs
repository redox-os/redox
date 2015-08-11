use core::clone::Clone;
use core::mem::size_of;

use alloc::boxed::*;

use common::memory::*;
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
    fn request(&mut self, session: &Session, url: &URL, callback: Box<FnBox(String)>){
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

            let node = unfs.node(path.clone());

            if node as usize > 0{
                if (*node).data_sector_list.address > 0 {
                    let sector_list_ptr = alloc(size_of::<SectorList>()) as *mut SectorList;
                    if sector_list_ptr as usize > 0 {
                        let sector_list = &mut *sector_list_ptr;
                        unfs.disk.read((*node).data_sector_list.address, 1, sector_list_ptr as usize);

                        for i in 0..1 {
                            if sector_list.extents[i].block.address > 0 && sector_list.extents[i].length > 0{
                                session.request(&URL::from_string("ide:///".to_string() + sector_list.extents[i].block.address as usize + "/" + sector_list.extents[i].length as usize), callback);
                                break;
                            }
                        }
                        unalloc(sector_list_ptr as usize);
                    }
                }

                unalloc(node as usize);
            }else{
                let mut ret = String::new();

                for file in unfs.list(path.clone()).iter() {
                    if ret.len() > 0 {
                        ret = ret + "\n" + file.clone();
                    }else{
                        ret = file.clone();
                    }
                }

                callback(ret);
            }
        }
    }
}
