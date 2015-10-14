use core::ptr;

use common::memory;

/// A memory page
pub struct Page {
    /// The virtual address
    virtual_address: usize,
}

impl Page {
    /// Initialize the memory page
    pub unsafe fn init() {
        for table_i in 0..memory::PAGE_TABLE_SIZE {
            ptr::write((memory::PAGE_DIRECTORY + table_i * 4) as *mut u32,
                       (memory::PAGE_TABLES + table_i * memory::PAGE_TABLE_SIZE * 4) as u32 | 1);

            for entry_i in 0..memory::PAGE_TABLE_SIZE {
                Page::new((table_i * memory::PAGE_TABLE_SIZE + entry_i) * memory::PAGE_SIZE)
                    .map_identity();
            }
        }

        asm!("mov cr3, $0
            mov $0, cr0
            or $0, $1
            mov cr0, $0"
            :
            : "r"(memory::PAGE_DIRECTORY), "r"(0x80000000 as usize)
            : "memory"
            : "intel", "volatile");
    }

    /// Create a new memory page from a virtual address
    pub fn new(virtual_address: usize) -> Self {
        Page { virtual_address: virtual_address }
    }

    /// Get the entry address
    fn entry_address(&self) -> usize {
        let page = self.virtual_address / memory::PAGE_SIZE;
        let table = page / memory::PAGE_TABLE_SIZE;
        let entry = page % memory::PAGE_TABLE_SIZE;

        memory::PAGE_TABLES + (table * memory::PAGE_TABLE_SIZE + entry) * 4
    }

    /// Flush the memory page
    unsafe fn flush(&self) {
        asm!("invlpg [$0]"
            :
            : "{eax}"(self.virtual_address)
            : "memory"
            : "intel", "volatile");
    }

    /// Get the current physical address
    pub fn phys_addr(&self) -> usize {
        unsafe { (ptr::read(self.entry_address() as *mut u32) & 0xFFFFF000) as usize }
    }

    /// Get the current virtual address
    pub fn virt_addr(&self) -> usize {
        self.virtual_address & 0xFFFFF000
    }

    /// Map the memory page to a given physical memory address
    pub unsafe fn map(&mut self, physical_address: usize) {
        ptr::write(self.entry_address() as *mut u32,
                   (physical_address as u32 & 0xFFFFF000) | 1);
        self.flush();
    }

    /// Map to the virtual address
    pub unsafe fn map_identity(&mut self) {
        let physical_address = self.virtual_address;
        self.map(physical_address);
    }

    /// Unmap the memory page
    pub unsafe fn unmap(&mut self) {
        ptr::write(self.entry_address() as *mut u32, 0);
        self.flush();
    }
}
