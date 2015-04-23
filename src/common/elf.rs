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
            if entry >= 0xC0000000 && entry < 0xC0000000 + 1024*4096 {
                // Setup 4 MB upper mem space to map to program
                for i in 0..1024 {
                    let virtual_address = 0xC0000000 + i*4096;
                    let physical_address = self.data + i*4096 + 4096;
                    set_page(virtual_address, physical_address/*extra 4096 to handle null segment*/);
                }
                
                asm!("call $0\n"
                    : : "{eax}"(entry) : : "intel");
                
                // Reset 4 MB upper mem space to identity
                for i in 0..1024 {
                    identity_page(0xC0000000 + i*4096);
                }
            }else{
                d("Empty ELF Entry\n");
            }
        }else{
            d("Empty ELF Data\n");
        }
    }
}