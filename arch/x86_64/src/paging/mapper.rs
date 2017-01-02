use core::ptr::Unique;

use memory::{allocate_frame, deallocate_frame, Frame};

use super::{Page, PAGE_SIZE, PhysicalAddress, VirtualAddress};
use super::entry::{self, EntryFlags};
use super::table::{self, Table, Level4};

pub struct Mapper {
    p4: Unique<Table<Level4>>,
}

impl Mapper {
    /// Create a new page table
    pub unsafe fn new() -> Mapper {
        Mapper {
            p4: Unique::new(table::P4),
        }
    }

    pub fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.get() }
    }

    pub fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.get_mut() }
    }

    /// Map a page to a frame
    pub fn map_to(&mut self, page: Page, frame: Frame, flags: EntryFlags) {
        let mut p3 = self.p4_mut().next_table_create(page.p4_index());
        let mut p2 = p3.next_table_create(page.p3_index());
        let mut p1 = p2.next_table_create(page.p2_index());

        assert!(p1[page.p1_index()].is_unused(),
            "{:X}: Set to {:X}: {:?}, requesting {:X}: {:?}",
            page.start_address().get(),
            p1[page.p1_index()].address().get(), p1[page.p1_index()].flags(),
            frame.start_address().get(), flags);
        p1[page.p1_index()].set(frame, flags | entry::PRESENT);
    }

    /// Map a page to the next free frame
    pub fn map(&mut self, page: Page, flags: EntryFlags) {
        let frame = allocate_frame().expect("out of frames");
        self.map_to(page, frame, flags)
    }

    /// Update flags for a page
    pub fn remap(&mut self, page: Page, flags: EntryFlags) {
        let mut p3 = self.p4_mut().next_table_mut(page.p4_index()).expect("failed to remap: no p3");
        let mut p2 = p3.next_table_mut(page.p3_index()).expect("failed to remap: no p2");
        let mut p1 = p2.next_table_mut(page.p2_index()).expect("failed to remap: no p1");
        let frame = p1[page.p1_index()].pointed_frame().expect("failed to remap: not mapped");
        p1[page.p1_index()].set(frame, flags | entry::PRESENT);
    }

    /// Identity map a frame
    pub fn identity_map(&mut self, frame: Frame, flags: EntryFlags) {
        let page = Page::containing_address(VirtualAddress::new(frame.start_address().get()));
        self.map_to(page, frame, flags)
    }

    /// Unmap a page
    pub fn unmap(&mut self, page: Page) {
        let p1 = self.p4_mut()
                     .next_table_mut(page.p4_index())
                     .and_then(|p3| p3.next_table_mut(page.p3_index()))
                     .and_then(|p2| p2.next_table_mut(page.p2_index()))
                     .expect("unmap does not support huge pages");
        let frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();
        // TODO free p(1,2,3) table if empty
        deallocate_frame(frame);
    }

    /// Unmap a page, return frame without free
    pub fn unmap_return(&mut self, page: Page) -> Frame {
        let p1 = self.p4_mut()
                     .next_table_mut(page.p4_index())
                     .and_then(|p3| p3.next_table_mut(page.p3_index()))
                     .and_then(|p2| p2.next_table_mut(page.p2_index()))
                     .expect("unmap_return does not support huge pages");
        let frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();
        frame
    }

    pub fn translate_page(&self, page: Page) -> Option<Frame> {
        self.p4().next_table(page.p4_index())
            .and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].pointed_frame())
    }

    pub fn translate_page_flags(&self, page: Page) -> Option<EntryFlags> {
        self.p4().next_table(page.p4_index())
            .and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| Some(p1[page.p1_index()].flags()))
    }

    /// Translate a virtual address to a physical one
    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virtual_address.get() % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| PhysicalAddress::new(frame.start_address().get() + offset))
    }
}
