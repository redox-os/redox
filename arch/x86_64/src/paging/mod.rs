//! # Paging
//! Some code was borrowed from [Phil Opp's Blog](http://os.phil-opp.com/modifying-page-tables.html)

use core::mem;
use core::ops::{Deref, DerefMut};
use x86::{msr, tlb};

use memory::{allocate_frame, Frame};

use self::entry::{EntryFlags, PRESENT, WRITABLE, NO_EXECUTE};
use self::mapper::Mapper;
use self::temporary_page::TemporaryPage;

pub mod entry;
pub mod mapper;
pub mod table;
pub mod temporary_page;

/// Number of entries per page table
pub const ENTRY_COUNT: usize = 512;

/// Size of pages
pub const PAGE_SIZE: usize = 4096;

/// Initialize paging
///
/// Returns page table and thread control block offset
pub unsafe fn init(stack_start: usize, stack_end: usize) -> (ActivePageTable, usize) {
    extern {
        /// The starting byte of the text (code) data segment.
        static mut __text_start: u8;
        /// The ending byte of the text (code) data segment.
        static mut __text_end: u8;
        /// The starting byte of the _.rodata_ (read-only data) segment.
        static mut __rodata_start: u8;
        /// The ending byte of the _.rodata_ (read-only data) segment.
        static mut __rodata_end: u8;
        /// The starting byte of the _.data_ segment.
        static mut __data_start: u8;
        /// The ending byte of the _.data_ segment.
        static mut __data_end: u8;
        /// The starting byte of the thread data segment
        static mut __tdata_start: u8;
        /// The ending byte of the thread data segment
        static mut __tdata_end: u8;
        /// The starting byte of the thread BSS segment
        static mut __tbss_start: u8;
        /// The ending byte of the thread BSS segment
        static mut __tbss_end: u8;
        /// The starting byte of the _.bss_ (uninitialized data) segment.
        static mut __bss_start: u8;
        /// The ending byte of the _.bss_ (uninitialized data) segment.
        static mut __bss_end: u8;
    }

    let mut active_table = ActivePageTable::new();

    let mut temporary_page = TemporaryPage::new(Page::containing_address(VirtualAddress::new(0x8_0000_0000)));

    let mut new_table = {
        let frame = allocate_frame().expect("no more frames in paging::init new_table");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        {
            let mut remap = |start: usize, end: usize, flags: EntryFlags| {
                if end > start {
                    let start_frame = Frame::containing_address(PhysicalAddress::new(start));
                    let end_frame = Frame::containing_address(PhysicalAddress::new(end - 1));
                    for frame in Frame::range_inclusive(start_frame, end_frame) {
                        mapper.identity_map(frame.clone(), flags);

                        //let page = Page::containing_address(VirtualAddress::new(frame.start_address().get() + ::KERNEL_OFFSET));
                        //mapper.map_to(page, frame, flags);
                    }
                }
            };

            // Remap stack writable, no execute
            remap(stack_start, stack_end, PRESENT | NO_EXECUTE | WRITABLE);

            // Remap a section with `flags`
            let mut remap_section = |start: &u8, end: &u8, flags: EntryFlags| {
                remap(start as *const _ as usize, end as *const _ as usize, flags);
            };
            // Remap text read-only
            remap_section(& __text_start, & __text_end, PRESENT);
            // Remap rodata read-only, no execute
            remap_section(& __rodata_start, & __rodata_end, PRESENT | NO_EXECUTE);
            // Remap data writable, no execute
            remap_section(& __data_start, & __data_end, PRESENT | NO_EXECUTE | WRITABLE);
            // Remap bss writable, no execute
            remap_section(& __bss_start, & __bss_end, PRESENT | NO_EXECUTE | WRITABLE);
        }
    });

    let uncacheable = 0;
    let write_combining = 1;
    let write_through = 4;
    //let write_protected = 5;
    let write_back = 6;
    let uncached = 7;

    let pat0 = write_back;
    let pat1 = write_through;
    let pat2 = uncached;
    let pat3 = uncacheable;

    let pat4 = write_combining;
    let pat5 = pat1;
    let pat6 = pat2;
    let pat7 = pat3;

    msr::wrmsr(msr::IA32_PAT, pat7 << 56 | pat6 << 48 | pat5 << 40 | pat4 << 32
                            | pat3 << 24 | pat2 << 16 | pat1 << 8 | pat0);

    active_table.switch(new_table);

    // Map and copy TDATA
    {
        let start = & __tdata_start as *const _ as usize;
        let end = & __tdata_end as *const _ as usize;
        if end > start {
            temporary_page.map(allocate_frame().expect("no more frames in paging::init TDATA"), PRESENT | NO_EXECUTE | WRITABLE, &mut active_table);

            let start_page = Page::containing_address(VirtualAddress::new(start));
            let end_page = Page::containing_address(VirtualAddress::new(end - 1));
            for page in Page::range_inclusive(start_page, end_page) {
                // Copy parent to temporary page
                {
                    let frame = Frame::containing_address(PhysicalAddress::new(page.start_address().get()));
                    active_table.identity_map(frame, PRESENT | NO_EXECUTE);
                    active_table.flush(page);
                    ::externs::memcpy(temporary_page.start_address().get() as *mut u8, page.start_address().get() as *const u8, 4096);
                    active_table.unmap(page);
                }
                // Copy temporary page to child
                {
                    active_table.map(page, PRESENT | NO_EXECUTE | WRITABLE);
                    active_table.flush(page);
                    ::externs::memcpy(page.start_address().get() as *mut u8, temporary_page.start_address().get() as *const u8, 4096);
                }
            }

            temporary_page.unmap(&mut active_table);
        }
    }

    // Map and clear TBSS
    let tcb_offset;
    {
        let start = & __tbss_start as *const _ as usize;
        let end = & __tbss_end as *const _ as usize;
        tcb_offset = end - mem::size_of::<usize>();
        if end > start {
            let start_page = Page::containing_address(VirtualAddress::new(start));
            let end_page = Page::containing_address(VirtualAddress::new(end - 1));
            for page in Page::range_inclusive(start_page, end_page) {
                active_table.map(page, PRESENT | NO_EXECUTE | WRITABLE);
                active_table.flush(page);
                ::externs::memset(page.start_address().get() as *mut u8, 0, 4096);
            }

            *(tcb_offset as *mut usize) = end;
        }
    }

    active_table.flush_all();

    (active_table, tcb_offset)
}

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        use x86::controlregs;

        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(
                PhysicalAddress::new(unsafe { controlregs::cr3() } as usize)
            ),
        };
        unsafe {
            controlregs::cr3_write(new_table.p4_frame.start_address().get() as u64);
        }
        old_table
    }

    pub fn flush(&mut self, page: Page) {
        unsafe { tlb::flush(page.start_address().get()); }
    }

    pub fn flush_all(&mut self) {
        unsafe { tlb::flush_all(); }
    }

    pub fn with<F>(&mut self, table: &mut InactivePageTable, temporary_page: &mut temporary_page::TemporaryPage, f: F)
        where F: FnOnce(&mut Mapper)
    {
        use x86::controlregs;

        {
            let backup = Frame::containing_address(PhysicalAddress::new(unsafe { controlregs::cr3() as usize }));

            // map temporary_page to current p4 table
            let p4_table = temporary_page.map_table_frame(backup.clone(), PRESENT | WRITABLE | NO_EXECUTE, self);

            // overwrite recursive mapping
            self.p4_mut()[511].set(table.p4_frame.clone(), PRESENT | WRITABLE | NO_EXECUTE);
            self.flush_all();

            // execute f in the new context
            f(self);

            // restore recursive mapping to original p4 table
            p4_table[511].set(backup, PRESENT | WRITABLE | NO_EXECUTE);
            self.flush_all();
        }

        temporary_page.unmap(self);
    }

    pub unsafe fn address(&self) -> usize {
        use x86::controlregs;
        controlregs::cr3() as usize
    }
}

pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(frame: Frame, active_table: &mut ActivePageTable, temporary_page: &mut TemporaryPage) -> InactivePageTable {
        {
            let table = temporary_page.map_table_frame(frame.clone(), PRESENT | WRITABLE | NO_EXECUTE, active_table);
            // now we are able to zero the table
            table.zero();
            // set up recursive mapping for the table
            table[511].set(frame.clone(), PRESENT | WRITABLE | NO_EXECUTE);
        }
        temporary_page.unmap(active_table);

        InactivePageTable { p4_frame: frame }
    }

    pub unsafe fn address(&self) -> usize {
        self.p4_frame.start_address().get()
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    number: usize
}

impl Page {
    pub fn start_address(&self) -> VirtualAddress {
        VirtualAddress::new(self.number * PAGE_SIZE)
    }

    pub fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    pub fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    pub fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    pub fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }

    pub fn containing_address(address: VirtualAddress) -> Page {
        //TODO assert!(address.get() < 0x0000_8000_0000_0000 || address.get() >= 0xffff_8000_0000_0000,
        //    "invalid address: 0x{:x}", address.get());
        Page { number: address.get() / PAGE_SIZE }
    }

    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter {
            start: start,
            end: end,
        }
    }
}

pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let page = self.start;
            self.start.number += 1;
            Some(page)
        } else {
            None
        }
    }
}
