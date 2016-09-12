use arch::externs::memset;
use arch::paging::{ActivePageTable, Page, PageIter, VirtualAddress};
use arch::paging::entry::EntryFlags;

#[derive(Debug)]
pub struct Memory {
    start: VirtualAddress,
    size: usize,
    flags: EntryFlags
}

impl Memory {
    pub fn new(start: VirtualAddress, size: usize, flags: EntryFlags, flush: bool, clear: bool) -> Self {
        let mut memory = Memory {
            start: start,
            size: size,
            flags: flags
        };

        memory.map(flush, clear);

        memory
    }

    pub fn start_address(&self) -> VirtualAddress {
        self.start
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn pages(&self) -> PageIter {
        let start_page = Page::containing_address(self.start);
        let end_page = Page::containing_address(VirtualAddress::new(self.start.get() + self.size - 1));
        Page::range_inclusive(start_page, end_page)
    }

    pub fn map(&mut self, flush: bool, clear: bool) {
        let mut active_table = unsafe { ActivePageTable::new() };

        //TODO: Clear pages?
        for page in self.pages() {
            active_table.map(page, self.flags);
            if flush {
                active_table.flush(page);
            }
        }

        if clear {
            assert!(flush);
            unsafe { memset(self.start_address().get() as *mut u8, 0, self.size); }
        }
    }

    pub fn unmap(&mut self, flush: bool) {
        let mut active_table = unsafe { ActivePageTable::new() };

        for page in self.pages() {
            active_table.unmap(page);
            if flush {
                active_table.flush(page);
            }
        }
    }

    pub fn remap(&mut self, new_flags: EntryFlags, flush: bool) {
        let mut active_table = unsafe { ActivePageTable::new() };

        for page in self.pages() {
            active_table.remap(page, new_flags);
            if flush {
                active_table.flush(page);
            }
        }

        self.flags = new_flags;
    }

    pub fn resize(&mut self, new_size: usize, flush: bool, clear: bool) {
        let mut active_table = unsafe { ActivePageTable::new() };

        //TODO: Calculate page changes to minimize operations
        if new_size > self.size {
            let start_page = Page::containing_address(VirtualAddress::new(self.start.get() + self.size));
            let end_page = Page::containing_address(VirtualAddress::new(self.start.get() + new_size - 1));
            for page in Page::range_inclusive(start_page, end_page) {
                if active_table.translate_page(page).is_none() {
                    active_table.map(page, self.flags);
                    if flush {
                        active_table.flush(page);
                    }
                }
            }

            if clear {
                assert!(flush);
                unsafe { memset((self.start.get() + self.size) as *mut u8, 0, new_size - self.size); }
            }
        } else if new_size < self.size {
            let start_page = Page::containing_address(VirtualAddress::new(self.start.get() + new_size));
            let end_page = Page::containing_address(VirtualAddress::new(self.start.get() + self.size - 1));
            for page in Page::range_inclusive(start_page, end_page) {
                if active_table.translate_page(page).is_some() {
                    active_table.unmap(page);
                    if flush {
                        active_table.flush(page);
                    }
                }
            }
        }

        self.size = new_size;
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        self.unmap(true);
    }
}
