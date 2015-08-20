use common::pci::*;

use programs::common::*;

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

        let mut i = 0;
        for part in url.path.split("\n".to_string()) {
            match i {
                0 => bus = part.to_num() as isize,
                1 => slot = part.to_num() as isize,
                2 => func = part.to_num() as isize,
                3 => reg = part.to_num() as isize,
                _ => ()
            }
            i += 1;
        }

        if bus >= 0 && bus < 256 && slot >= 0 && slot < 32 && func >= 0 && func < 8 && reg >= 0 && reg < 256 {
            let data;
            unsafe {
                data = pci_read(bus as usize, slot as usize, func as usize, reg as usize);
            }

            return box VecResource::new(ResourceType::File, String::from_num(data).to_utf8());
        }else{
            return box NoneResource;
        }
    }
}
