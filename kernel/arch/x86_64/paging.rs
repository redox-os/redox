use core::ptr;

// PAGE_LEVEL_4:
// 512 qwords pointing to page directory pointers
// PAGE_DIR_PTRS:
// 512 qwords pointing to page directories
// PAGE_DIRECTORIES:
// 512 qwords pointing to page tables
// PAGE_TABLES:
// 512 * 512 qwords pointing to pages
// PAGE_END:
//

pub const PAGE_TABLE_SIZE: usize = 512;
pub const PAGE_ENTRY_SIZE: usize = 8;
pub const PAGE_SIZE: usize = 4096;

pub const PAGE_LEVEL_4: usize = 0x200000;
pub const PAGE_DIR_PTRS: usize = PAGE_LEVEL_4 + PAGE_TABLE_SIZE * PAGE_ENTRY_SIZE;
pub const PAGE_DIRECTORIES: usize = PAGE_DIR_PTRS + PAGE_TABLE_SIZE * PAGE_ENTRY_SIZE;
pub const PAGE_TABLES: usize = PAGE_DIRECTORIES + 4 * PAGE_TABLE_SIZE * PAGE_ENTRY_SIZE;
pub const PAGE_END: usize = PAGE_TABLES + 4 * PAGE_TABLE_SIZE * PAGE_TABLE_SIZE * PAGE_ENTRY_SIZE;

/// A memory page
pub struct Page {
    /// The virtual address
    virtual_address: usize,
}

impl Page {
    /// Initialize the memory page
    pub unsafe fn init() {
        for l4_i in 0..PAGE_TABLE_SIZE {
            if l4_i == 0 {
                ptr::write((PAGE_LEVEL_4 + l4_i * PAGE_ENTRY_SIZE) as *mut u64,
                           (PAGE_DIR_PTRS + l4_i * PAGE_TABLE_SIZE * PAGE_ENTRY_SIZE) as u64 |
                           1 << 2 | 0b11 << 1 | 1); //Allow userspace, read/write, present
            } else {
                ptr::write((PAGE_LEVEL_4 + l4_i * PAGE_ENTRY_SIZE) as *mut u64, 0);
            }
        }

        for dp_i in 0..PAGE_TABLE_SIZE {
            if dp_i < 4 {
                ptr::write((PAGE_DIR_PTRS + dp_i * PAGE_ENTRY_SIZE) as *mut u64,
                           (PAGE_DIRECTORIES + dp_i * PAGE_TABLE_SIZE * PAGE_ENTRY_SIZE) as u64 |
                           1 << 2 |
                           0b11 << 1 | 1); //Allow userspace, read/write, present
            } else {
                ptr::write((PAGE_DIR_PTRS + dp_i * PAGE_ENTRY_SIZE) as *mut u64, 0);
            }
        }

        for table_i in 0..4 * PAGE_TABLE_SIZE {
            ptr::write((PAGE_DIRECTORIES + table_i * PAGE_ENTRY_SIZE) as *mut u64,
                       (PAGE_TABLES + table_i * PAGE_TABLE_SIZE * PAGE_ENTRY_SIZE) as u64 |
                       1 << 2 | 0b11 << 1 | 1); //Allow userspace, read/write, present

            for entry_i in 0..PAGE_TABLE_SIZE {
                Page::new((table_i * PAGE_TABLE_SIZE + entry_i) * PAGE_SIZE).map_identity();
            }
        }

        asm!("mov cr3, $0
            mov $0, cr0
            or $0, $1
            mov cr0, $0"
            :
            : "r"(PAGE_LEVEL_4), "r"(0x80000000 as usize)
            : "memory"
            : "intel", "volatile");
    }

    /// Create a new memory page from a virtual address
    pub fn new(virtual_address: usize) -> Self {
        Page { virtual_address: virtual_address }
    }

    /// Get the entry address
    fn entry_address(&self) -> usize {
        let page = self.virtual_address / PAGE_SIZE;
        let table = page / PAGE_TABLE_SIZE;
        let entry = page % PAGE_TABLE_SIZE;

        PAGE_TABLES + (table * PAGE_TABLE_SIZE + entry) * PAGE_ENTRY_SIZE
    }

    /// Flush the memory page
    unsafe fn flush(&self) {
        asm!("invlpg [$0]"
            :
            : "{rax}"(self.virtual_address)
            : "memory"
            : "intel", "volatile");
    }

    /// Get the current physical address
    pub fn phys_addr(&self) -> usize {
        unsafe { (ptr::read(self.entry_address() as *mut u64) & 0xFFFFFFFFFFFFF000) as usize }
    }

    /// Get the current virtual address
    pub fn virt_addr(&self) -> usize {
        self.virtual_address & 0xFFFFFFFFFFFFF000
    }

    /// Map the memory page to a given physical memory address
    pub unsafe fn map(&mut self, physical_address: usize) {
        ptr::write(self.entry_address() as *mut u64,
                   (physical_address as u64 & 0xFFFFFFFFFFFFF000) | 1); //present
        self.flush();
    }

    /// Map the memory page to a given physical memory address and allow userspace read access
    pub unsafe fn map_user_read(&mut self, physical_address: usize) {
        ptr::write(self.entry_address() as *mut u64,
                   (physical_address as u64 & 0xFFFFFFFFFFFFF000) | 1 << 2 | 1); //Allow userspace, present
        self.flush();
    }

    /// Map the memory page to a given physical memory address and allow userspace read/write access
    pub unsafe fn map_user_write(&mut self, physical_address: usize) {
        ptr::write(self.entry_address() as *mut u64,
                   (physical_address as u64 & 0xFFFFFFFFFFFFF000) | 1 << 2 | 1 << 1 | 1); //Allow userspace, read/write, present
        self.flush();
    }

    /// Map to the virtual address
    pub unsafe fn map_identity(&mut self) {
        let physical_address = self.virtual_address;
        self.map(physical_address);
    }

    /// Unmap the memory page
    pub unsafe fn unmap(&mut self) {
        ptr::write(self.entry_address() as *mut u64, 0);
        self.flush();
    }
}
