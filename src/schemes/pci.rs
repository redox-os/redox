use core::clone::Clone;
use core::option::Option;

use alloc::boxed::*;

use common::pci::*;
use common::string::*;
use common::url::*;

use programs::session::*;

pub struct PCIScheme;

impl SessionModule for PCIScheme {
    fn scheme(&self) -> String {
        return "pci".to_string();
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
