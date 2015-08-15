use core::clone::Clone;
use core::mem::size_of;

use alloc::boxed::*;

use common::memory::*;
use common::resource::*;
use common::string::*;
use common::url::*;
use common::vec::*;

use filesystems::unfs::*;

use programs::session::*;
use programs::syscall;

pub struct FileScheme;

impl SessionModule for FileScheme {
    fn scheme(&self) -> String {
        return "file".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource>{
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

            let mut ret: Box<Resource> = box NoneResource;

            let node = unfs.node(path.clone());

            if node as usize > 0{
                if (*node).data_sector_list.address > 0 {
                    let sector_list_ptr = alloc(size_of::<SectorList>()) as *mut SectorList;
                    if sector_list_ptr as usize > 0 {
                        let sector_list = &mut *sector_list_ptr;
                        unfs.disk.read((*node).data_sector_list.address, 1, sector_list_ptr as usize);

                        for i in 0..1 {
                            if sector_list.extents[i].block.address > 0 && sector_list.extents[i].length > 0{
                                ret = syscall::open(&URL::from_string("ide:///".to_string() + sector_list.extents[i].block.address as usize + "/" + sector_list.extents[i].length as usize));
                                break;
                            }
                        }

                        unalloc(sector_list_ptr as usize);
                    }
                }

                unalloc(node as usize);
            }else{
                let mut list = String::new();

                for file in unfs.list(path.clone()).iter() {
                    if list.len() > 0 {
                        list = list + "\n" + file.clone();
                    }else{
                        list = file.clone();
                    }
                }

                ret = box VecResource::new(list.to_utf8());
            }

            return ret;
        }
    }

    fn open_async(&mut self, url: &URL, callback: Box<FnBox(Box<Resource>)>){
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

                        if sector_list.extents[0].block.address > 0 && sector_list.extents[0].length > 0{
                            syscall::open_async(&URL::from_string("ide:///".to_string() + sector_list.extents[0].block.address as usize + "/" + sector_list.extents[0].length as usize), callback);
                        }else{
                            callback(box NoneResource);
                        }

                        unalloc(sector_list_ptr as usize);
                    }else{
                        callback(box NoneResource);
                    }
                }else{
                    callback(box NoneResource);
                }

                unalloc(node as usize);
            }else{
                let mut list = String::new();

                for file in unfs.list(path.clone()).iter() {
                    if list.len() > 0 {
                        list = list + "\n" + file.clone();
                    }else{
                        list = file.clone();
                    }
                }

                callback(box VecResource::new(list.to_utf8()));
            }
        }
    }
}
