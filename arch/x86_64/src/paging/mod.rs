//! # Paging
//! Some code was borrowed from [Phil Opp's Blog](http://os.phil-opp.com/modifying-page-tables.html)

use core::ptr::Unique;

use memory::{Frame, FrameAllocator};

use self::entry::EntryFlags;
use self::table::{Table, Level4};

pub mod entry;
pub mod table;

/// Number of entries per page table
pub const ENTRY_COUNT: usize = 512;

/// Size of pages
pub const PAGE_SIZE: usize = 4096;

/// Initialize paging
pub unsafe fn init() -> ActivePageTable {
    ActivePageTable::new()
}

pub struct ActivePageTable {
    p4: Unique<Table<Level4>>,
}

impl ActivePageTable {
    /// Create a new page table
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            p4: Unique::new(table::P4),
        }
    }

    fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.get() }
    }

    fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.get_mut() }
    }

    /// Map a page to a frame
    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
        let mut p3 = self.p4_mut().next_table_create(page.p4_index(), allocator);
        let mut p2 = p3.next_table_create(page.p3_index(), allocator);
        let mut p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | entry::PRESENT);
    }

    /// Map a page to the next free frame
    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
        let frame = allocator.allocate_frame().expect("out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    /// Identity map a frame
    pub fn identity_map<A>(&mut self, frame: Frame, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
        let page = Page::containing_address(VirtualAddress::new(frame.start_address().get()));
        self.map_to(page, frame, flags, allocator)
    }

    /// Unmap a page
    fn unmap<A>(&mut self, page: Page, allocator: &mut A)
        where A: FrameAllocator
    {
        assert!(self.translate(page.start_address()).is_some());

        let p1 = self.p4_mut()
                     .next_table_mut(page.p4_index())
                     .and_then(|p3| p3.next_table_mut(page.p3_index()))
                     .and_then(|p2| p2.next_table_mut(page.p2_index()))
                     .expect("mapping code does not support huge pages");
        let frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();
        // TODO free p(1,2,3) table if empty
        allocator.deallocate_frame(frame);
    }

    fn translate_page(&self, page: Page) -> Option<Frame> {
        self.p4().next_table(page.p4_index())
            .and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].pointed_frame())
    }

    /// Translate a virtual address to a physical one
    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virtual_address.get() % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| PhysicalAddress::new(frame.start_address().get() + offset))
    }
}

/// A physical address.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicalAddress(usize);

impl PhysicalAddress {
    pub fn new(address: usize) -> Self {
        PhysicalAddress(address)
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

/// A virtual address.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
    pub fn new(address: usize) -> Self {
        VirtualAddress(address)
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

/// Page
#[derive(Debug, Clone, Copy)]
pub struct Page {
    number: usize
}

impl Page {
    fn start_address(&self) -> VirtualAddress {
        VirtualAddress::new(self.number * PAGE_SIZE)
    }

    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }

    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(address.get() < 0x0000_8000_0000_0000 || address.get() >= 0xffff_8000_0000_0000,
            "invalid address: 0x{:x}", address.get());
        Page { number: address.get() / PAGE_SIZE }
    }
}
