use common::memory;

use core::ptr;

// PAGE_DIRECTORY:
// 1024 dwords pointing to page tables
// PAGE_TABLES:
// 1024 * 1024 dwords pointing to pages
// PAGE_END:
//

//Page flags
pub const PF_PRESENT: usize = 1;
pub const PF_WRITE: usize = 1 << 1;
pub const PF_USER: usize = 1 << 2;
pub const PF_WRITE_THROUGH: usize = 1 << 3;
pub const PF_CACHE_DISABLE: usize = 1 << 4;
pub const PF_ACCESSED: usize = 1 << 5;
pub const PF_DIRTY: usize = 1 << 6;
pub const PF_SIZE: usize = 1 << 7;
pub const PF_GLOBAL: usize = 1 << 8;
//Extra flags (Redox specific)
pub const PF_ALLOC: usize = 1 << 9;
pub const PF_EXEC: usize = 1 << 10;
pub const PF_STACK: usize = 1 << 11;

pub const PF_ALL: usize =  0xFFF;
pub const PF_NONE: usize = 0xFFFFF000;

pub struct Pager {
    directory: *mut PageDirectory
}

impl Pager {
    pub unsafe fn new() -> Pager {
        let directory = memory::alloc_type();
        ptr::write(directory, PageDirectory {
            tables: [0; 1024]
        });
        Pager {
            directory: directory
        }
    }

    pub unsafe fn map(&self) {
        asm!("mov cr3, $0"
            :
            : "r"(self.directory as u32)
            : "memory"
            : "intel", "volatile");
    }
}

impl Drop for Pager {
    fn drop(&mut self) {
        unsafe {
            drop(ptr::read(self.directory));
            memory::unalloc_type(self.directory);
        }
    }
}

#[repr(packed)]
pub struct PageDirectory {
    tables: [usize; 1024]
}

impl Drop for PageDirectory {
    fn drop(&mut self) {
        for table in self.tables.iter() {
            if table & PF_ALLOC == PF_ALLOC {
                unsafe {
                    drop(ptr::read((*table & PF_NONE) as *mut PageTable));
                    memory::unalloc(*table & PF_NONE);
                }
            }
        }
    }
}

#[repr(packed)]
pub struct PageTable {
    pages: [usize; 1024]
}

impl Drop for PageTable {
    fn drop(&mut self) {
        for page in self.pages.iter() {
            if page & PF_ALLOC == PF_ALLOC {
                unsafe { memory::unalloc(*page & PF_NONE) };
            }
        }
    }
}

pub const PAGE_TABLE_SIZE: usize = 1024;
pub const PAGE_ENTRY_SIZE: usize = 4;
pub const PAGE_SIZE: usize = 4096;

pub const PAGE_DIRECTORY: usize = 0x200000;
pub const PAGE_TABLES: usize = PAGE_DIRECTORY + PAGE_TABLE_SIZE * PAGE_ENTRY_SIZE;
pub const PAGE_END: usize = PAGE_TABLES + PAGE_TABLE_SIZE * PAGE_TABLE_SIZE * PAGE_ENTRY_SIZE;

/// A memory page
pub struct Page {
    /// The virtual address
    virtual_address: usize,
}

impl Page {
    /// Initialize the memory page
    pub unsafe fn init() {
        for table_i in 0..PAGE_TABLE_SIZE {
            ptr::write((PAGE_DIRECTORY + table_i * PAGE_ENTRY_SIZE) as *mut u32,
                       // TODO: Use more restrictive flags
                       (PAGE_TABLES + table_i * PAGE_TABLE_SIZE * PAGE_ENTRY_SIZE) as u32 |
                       1 << 2 | 1 << 1 | 1); //Allow userspace, read/write, present

            for entry_i in 0..PAGE_TABLE_SIZE {
                Page::new((table_i * PAGE_TABLE_SIZE + entry_i) * PAGE_SIZE).map_identity();
            }
        }

        asm!("mov cr3, $0
            mov $0, cr0
            or $0, $1
            mov cr0, $0"
            :
            : "r"(PAGE_DIRECTORY), "r"(0x80000000 as usize)
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
                   (physical_address as u32 & 0xFFFFF000) | 1); //present
        self.flush();
    }

    /// Map the memory page to a given physical memory address, and allow userspace read access
    pub unsafe fn map_user_read(&mut self, physical_address: usize) {
        ptr::write(self.entry_address() as *mut u32,
                   (physical_address as u32 & 0xFFFFF000) | 1 << 2 | 1); //Allow userspace, present
        self.flush();
    }

    /// Map the memory page to a given physical memory address, and allow userspace read/write access
    pub unsafe fn map_user_write(&mut self, physical_address: usize) {
        ptr::write(self.entry_address() as *mut u32,
                   (physical_address as u32 & 0xFFFFF000) | 1 << 2 | 1 << 1 | 1); //Allow userspace, read/write, present
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
