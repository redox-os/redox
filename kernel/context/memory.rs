use arch::paging::{ActivePageTable, Page, PageIter, VirtualAddress};
use arch::paging::entry::EntryFlags;

#[derive(Debug)]
pub struct Memory {
    start: VirtualAddress,
    size: usize,
    flags: EntryFlags
}

impl Memory {
    pub fn new(start: VirtualAddress, size: usize, flags: EntryFlags) -> Self {
        let mut memory = Memory {
            start: start,
            size: size,
            flags: flags
        };

        memory.map(true);

        memory
    }

    pub fn pages(&self) -> PageIter {
        let start_page = Page::containing_address(self.start);
        let end_page = Page::containing_address(VirtualAddress::new(self.start.get() + self.size - 1));
        Page::range_inclusive(start_page, end_page)
    }

    pub fn map(&mut self, flush: bool) {
        let mut active_table = unsafe { ActivePageTable::new() };

        for page in self.pages() {
            active_table.map(page, self.flags);
            if flush {
                active_table.flush(page);
            }
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

        self.flags = new_flags;

        for page in self.pages() {
            active_table.remap(page, self.flags);
            if flush {
                active_table.flush(page);
            }
        }
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        self.unmap(true);
    }
}
