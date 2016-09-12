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

        for page in self.pages() {
            active_table.remap(page, new_flags);
            if flush {
                active_table.flush(page);
            }
        }

        self.flags = new_flags;
    }

    pub fn resize(&mut self, new_size: usize, flush: bool) {
        let mut active_table = unsafe { ActivePageTable::new() };

        if new_size > self.size {
            let start_page = Page::containing_address(VirtualAddress::new(self.start.get() + self.size));
            let end_page = Page::containing_address(VirtualAddress::new(self.start.get() + new_size - 1));
            for page in Page::range_inclusive(start_page, end_page) {
                //println!("Map {:X}", page.start_address().get());
                if active_table.translate_page(page).is_none() {
                    //println!("Not found - mapping");
                    active_table.map(page, self.flags);
                    if flush {
                        active_table.flush(page);
                    }
                } else {
                    //println!("Found - skipping");
                }
            }
        } else if new_size < self.size {
            let start_page = Page::containing_address(VirtualAddress::new(self.start.get() + new_size));
            let end_page = Page::containing_address(VirtualAddress::new(self.start.get() + self.size - 1));
            for page in Page::range_inclusive(start_page, end_page) {
                //println!("Unmap {:X}", page.start_address().get());
                if active_table.translate_page(page).is_some() {
                    //println!("Found - unmapping");
                    active_table.unmap(page);
                    if flush {
                        active_table.flush(page);
                    }
                } else {
                    //println!("Not found - skipping");
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
