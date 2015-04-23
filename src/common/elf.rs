use common::debug::*;
use common::memory::*;

pub struct ELF {
    pub data: usize
}

impl ELF {
    pub unsafe fn new(file_data: usize) -> ELF {
        let data;
        if file_data > 0
            //Signature
            && *(file_data as *const u8) == 0x7F
            && *((file_data + 1) as *const u8) == 'E' as u8
            && *((file_data + 2) as *const u8) == 'L' as u8
            && *((file_data + 3) as *const u8) == 'F' as u8
            //1 for 32 bit, 2 for 64 bit
            && *((file_data + 4) as *const u8) == 1
            // TODO: Add more tests from header (architecture, platform)
        {
            data = file_data;
        }else{
            d("Invalid ELF Format\n");
            data = 0;
        }
    
        return ELF {
            data: data
        };
    }
    
    pub fn drop(&mut self){
        if self.data > 0 {
            unalloc(self.data);
            self.data = 0;
        }
    }
    
    pub unsafe fn run(&self) {
        if self.data > 0 {
            // TODO: Support 64-bit version
            let entry = *((self.data + 0x18) as *const u32);
            if entry > 0 {
                asm!("call $0\n"
                    : : "{eax}"(self.data as u32 + entry) : : "intel");
            }else{
                d("Empty ELF Entry\n");
            }
        }else{
            d("Empty ELF Data\n");
        }
    }
}