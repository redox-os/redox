use core::ptr;

use common::memory;

pub struct Page {
    virtual_address: usize
}

impl Page {
    pub unsafe fn init() {
        for table_i in 0..memory::PAGE_TABLE_SIZE {
            ptr::write((memory::PAGE_DIRECTORY + table_i * 4) as *mut u32, (memory::PAGE_TABLES + table_i * memory::PAGE_TABLE_SIZE * 4) as u32 | 1);

            for entry_i in 0..memory::PAGE_TABLE_SIZE {
                Page::new((table_i * memory::PAGE_TABLE_SIZE + entry_i) * memory::PAGE_SIZE).map_identity();
            }
        }

        asm!("mov cr3, $0\n
            mov $0, cr0\n
            or $0, 0x80000000\n
            mov cr0, $0\n"
            :
            : "{eax}"(memory::PAGE_DIRECTORY)
            : "memory"
            : "intel", "volatile");
    }

    pub fn new(virtual_address: usize) -> Page {
        Page {
            virtual_address: virtual_address
        }
    }

    fn entry_address(&self) -> usize {
        let page = self.virtual_address / memory::PAGE_SIZE;
        let table = page / memory::PAGE_TABLE_SIZE;
        let entry = page % memory::PAGE_TABLE_SIZE;

        memory::PAGE_TABLES + (table * memory::PAGE_TABLE_SIZE + entry) * 4
    }

    unsafe fn flush(&self) {
        asm!("invlpg [$0]"
            :
            : "{eax}"(self.virtual_address)
            : "memory"
            : "intel", "volatile");
    }

    pub unsafe fn map(&mut self, physical_address: usize) {
        ptr::write(self.entry_address() as *mut u32, (physical_address as u32 & 0xFFFFF000) | 1);
        self.flush();
    }

    pub unsafe fn map_identity(&mut self){
        let physical_address = self.virtual_address;
        self.map(physical_address);
    }

    pub unsafe fn unmap(&mut self) {
        ptr::write(self.entry_address() as *mut u32, 0);
        self.flush();
    }
}
