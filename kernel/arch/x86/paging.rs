use arch::memory;

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
    directory: usize,
    flags: usize,
}

impl Pager {
    /// Create a new Pager for x86
    /// # Safety
    /// - Allocates and initializes a new page directory
    /// - *Will fail if memory allocation fails*
    pub fn new(flags: usize) -> Pager {
        let directory;
        unsafe {
            directory = memory::alloc_type();
            ptr::write(directory, PageDirectory::new());
        }
        Pager {
            directory: directory as usize | PF_ALLOC,
            flags: flags,
        }
    }

    /// Use this Pager for memory operations
    /// # Safety
    /// - Sets CR3 to the page directory location, ensuring that flags are removed
    /// - *Will fail if memory allocation failed in Pager::new()*
    pub unsafe fn enable(&self) {
        asm!("mov cr3, $0"
            :
            : "r"(self.directory & PF_NONE)
            : "memory"
            : "intel", "volatile");
    }

    /// Map a virtual address to a physical address
    /// # Safety
    /// - Calls PageDirectory::map() using a raw pointer
    /// - *Will fail if memory allocation failed in Pager::new()*
    pub unsafe fn map(&mut self, virtual_address: usize, physical_address: usize) {
        let directory_ptr = (self.directory & PF_NONE) as *mut PageDirectory;
        let directory = &mut *directory_ptr;
        directory.map(virtual_address, physical_address, self.flags);
    }

    /// Unmap a virtual address
    /// # Safety
    /// - Calls PageDirectory::unmap() using a raw pointer
    /// - *Will fail if memory allocation failed in Pager::new()*
    pub unsafe fn unmap(&mut self, virtual_address: usize) {
        let directory_ptr = (self.directory & PF_NONE) as *mut PageDirectory;
        let directory = &mut *directory_ptr;
        directory.unmap(virtual_address);
    }
}

impl Drop for Pager {
    /// Drop the Pager
    /// # Safety
    /// - Calls drop on a raw pointer
    /// - *Will fail if memory allocation failed in Pager::new()*
    /// - *CR3 should be set to a different pager before dropping*
    fn drop(&mut self) {
        if self.directory & PF_ALLOC == PF_ALLOC {
            unsafe {
                let directory_ptr = (self.directory & PF_NONE) as *mut PageDirectory;
                drop(ptr::read(directory_ptr));
                memory::unalloc_type(directory_ptr);
            }
        }
    }
}

#[repr(packed)]
pub struct PageDirectory {
    entries: [usize; 1024]
}

impl PageDirectory {
    /// Create a new and empty PageDirectory
    fn new() -> PageDirectory {
        PageDirectory {
            entries: [0; 1024]
        }
    }

    /// Map a virtual address to a physical address
    /// # Safety
    /// - Calls PageTable::map() using a raw pointer
    /// - *Will fail if memory allocation failed*
    unsafe fn map(&mut self, virtual_address: usize, physical_address: usize, flags: usize) {
        let entry = &mut self.entries[(virtual_address >> 22) & 1023];
        if *entry & PF_NONE == 0 {
            let table_ptr = memory::alloc_type();
            ptr::write(table_ptr, PageTable::new());
            *entry = table_ptr as usize | PF_ALLOC | PF_PRESENT;
        }

        let table_ptr = (*entry & PF_NONE) as *mut PageTable;
        let table = &mut *table_ptr;
        table.map(virtual_address, physical_address, flags);
    }

    /// Unmap a virtual address
    /// # Safety
    /// - Calls PageTable::unmap() using a raw pointer
    /// - *Will fail if memory allocation failed*
    unsafe fn unmap(&mut self, virtual_address: usize){
        let entry = &mut self.entries[(virtual_address >> 22) & 1023];
        if *entry & PF_NONE > 0 {
            let table_ptr = (*entry & PF_NONE) as *mut PageTable;
            let table = &mut *table_ptr;
            table.unmap(virtual_address);
        }
    }
}

impl Drop for PageDirectory {
    fn drop(&mut self) {
        for entry in self.entries.iter() {
            if *entry & PF_ALLOC == PF_ALLOC {
                unsafe {
                    let table_ptr = (*entry & PF_NONE) as *mut PageTable;
                    drop(ptr::read(table_ptr));
                    memory::unalloc_type(table_ptr);
                }
            }
        }
    }
}

#[repr(packed)]
pub struct PageTable {
    entries: [usize; 1024]
}

impl PageTable {
    /// Create a new and empty PageTable
    fn new() -> PageTable {
        PageTable {
            entries: [0; 1024]
        }
    }

    unsafe fn map(&mut self, virtual_address: usize, physical_address: usize, flags: usize) {
        let entry = &mut self.entries[(virtual_address >> 12) & 1023];
        if *entry & PF_ALLOC == PF_ALLOC {
            memory::unalloc(*entry & PF_NONE);
        }
        *entry = physical_address & PF_NONE | flags;
    }

    unsafe fn unmap(&mut self, virtual_address: usize){
        let entry = &mut self.entries[(virtual_address >> 12) & 1023];
        if *entry & PF_ALLOC == PF_ALLOC {
            memory::unalloc(*entry & PF_NONE);
        }
        *entry = 0;
    }
}

impl Drop for PageTable {
    fn drop(&mut self) {
        for entry in self.entries.iter() {
            if *entry & PF_ALLOC == PF_ALLOC {
                unsafe { memory::unalloc(*entry & PF_NONE) };
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
            ptr::write((PAGE_DIRECTORY + table_i * PAGE_ENTRY_SIZE) as *mut usize,
                       // TODO: Use more restrictive flags
                       (PAGE_TABLES + table_i * PAGE_TABLE_SIZE * PAGE_ENTRY_SIZE) |
                       PF_USER | PF_WRITE | PF_PRESENT); //Allow userspace, read/write, present

            for entry_i in 0..PAGE_TABLE_SIZE {
                let addr = (table_i * PAGE_TABLE_SIZE + entry_i) * PAGE_SIZE;
                Page::new(addr).map_kernel_write(addr);
            }
        }

        asm!("mov cr3, $0
            mov $0, cr0
            or $0, $1
            mov cr0, $0"
            :
            : "r"(PAGE_DIRECTORY), "r"(1 << 31 | 1 << 16)
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
        unsafe { (ptr::read(self.entry_address() as *mut usize) & PF_NONE) as usize }
    }

    /// Get the current virtual address
    pub fn virt_addr(&self) -> usize {
        self.virtual_address & PF_NONE
    }

    /// Map the memory page to a given physical memory address
    pub unsafe fn map_kernel_read(&mut self, physical_address: usize) {
        ptr::write(self.entry_address() as *mut usize,
                   (physical_address & PF_NONE) | PF_PRESENT);
        self.flush();
    }

    /// Map the memory page to a given physical memory address
    pub unsafe fn map_kernel_write(&mut self, physical_address: usize) {
        ptr::write(self.entry_address() as *mut usize,
                   (physical_address & PF_NONE) | PF_WRITE | PF_PRESENT);
        self.flush();
    }

    /// Map the memory page to a given physical memory address, and allow userspace read access
    pub unsafe fn map_user_read(&mut self, physical_address: usize) {
        ptr::write(self.entry_address() as *mut usize,
                   (physical_address & PF_NONE) | PF_USER | PF_PRESENT);
        self.flush();
    }

    /// Map the memory page to a given physical memory address, and allow userspace read/write access
    pub unsafe fn map_user_write(&mut self, physical_address: usize) {
        ptr::write(self.entry_address() as *mut usize,
                   (physical_address & PF_NONE) | PF_USER | PF_WRITE | PF_PRESENT);
        self.flush();
    }

    /// Unmap the memory page
    pub unsafe fn unmap(&mut self) {
        ptr::write(self.entry_address() as *mut usize, 0);
        self.flush();
    }
}
