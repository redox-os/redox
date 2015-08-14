use core::clone::Clone;
use core::option::Option;

use alloc::boxed::*;

use common::pci::*;
use common::resource::*;
use common::string::*;
use common::url::*;

use programs::session::*;

pub struct PCIScheme;

impl SessionModule for PCIScheme {
    fn scheme(&self) -> String {
        return "pci".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        let mut bus = -1;
        let mut slot = -1;
        let mut func = -1;
        let mut reg = -1;

        for i in 0..url.path.len() {
            match url.path.get(i){
                Option::Some(part) => match i {
                    0 => {
                        bus = part.to_num() as isize;
                    },
                    1 => {
                        slot = part.to_num() as isize;
                    },
                    2 => {
                        func = part.to_num() as isize;
                    },
                    3 => {
                        reg = part.to_num() as isize;
                    },
                    _ => ()
                },
                Option::None => ()
            }
        }

        if bus >= 0 && bus < 256 && slot >= 0 && slot < 32 && func >= 0 && func < 8 && reg >= 0 && reg < 256 {
            let data;
            unsafe {
                data = pci_read(bus as usize, slot as usize, func as usize, reg as usize);
            }

            return box VecResource::new(String::from_num(data).to_utf8());
        }else{
            return box NoneResource;
        }
    }

    #[allow(unused_variables)]
    fn request(&mut self, session: &Session, url: &URL, callback: Box<FnBox(String)>){
        let mut bus = -1;
        let mut slot = -1;
        let mut func = -1;
        let mut reg = String::new();

        for i in 0..url.path.len() {
            match url.path.get(i){
                Option::Some(part) => match i {
                    0 => {
                        bus = part.to_num() as isize;
                    },
                    1 => {
                        slot = part.to_num() as isize;
                    },
                    2 => {
                        func = part.to_num() as isize;
                    },
                    3 => {
                        reg = part.clone();
                    },
                    _ => ()
                },
                Option::None => ()
            }
        }

        let ret;
        if bus >= 0 {
            if slot >= 0 {
                if func >= 0 {
                    if reg.len() > 0 {
                        if reg == "class".to_string() {
                            unsafe {
                                ret = String::from_num_radix((pci_read(bus as usize, slot as usize, func as usize, 8) >> 24) & 0xFF, 16);
                            }
                        }else{
                            ret = "Unknown register ".to_string() + reg.clone();
                        }
                    }else{
                        ret = String::from_num(256);
                    }
                }else{
                    ret = String::from_num(8);
                }
            }else{
                ret = String::from_num(32);
            }
        }else{
            ret = String::from_num(256);
        }
        callback(ret);
    }
}
