//! ELF executables

use collections::String;

use core::str;

#[cfg(target_arch = "x86")]
use goblin::elf32::{header, program_header};

#[cfg(target_arch = "x86_64")]
use goblin::elf64::{header, program_header};

use arch::externs::{memcpy, memset};
use arch::paging::{entry, ActivePageTable, Page, VirtualAddress};
use arch::start::usermode;
use arch::x86::tlb;

/// An ELF executable
pub struct Elf<'a> {
    pub data: &'a [u8],
    header: &'a header::Header
}

impl<'a> Elf<'a> {
    /// Create a ELF executable from data
    pub fn from(data: &'a [u8]) -> Result<Elf<'a>, String> {
        if data.len() < header::SIZEOF_EHDR {
            Err(format!("Elf: Not enough data: {} < {}", data.len(), header::SIZEOF_EHDR))
        } else if &data[..header::SELFMAG] != header::ELFMAG {
            Err(format!("Elf: Invalid magic: {:?} != {:?}", &data[..4], header::ELFMAG))
        } else if data.get(header::EI_CLASS) != Some(&header::ELFCLASS) {
            Err(format!("Elf: Invalid architecture: {:?} != {:?}", data.get(header::EI_CLASS), header::ELFCLASS))
        } else {
            Ok(Elf {
                data: data,
                header: unsafe { &*(data.as_ptr() as usize as *const header::Header) }
            })
        }
    }

    pub fn segments(&'a self) -> ElfSegments<'a> {
        ElfSegments {
            data: self.data,
            header: self.header,
            i: 0
        }
    }

    /// Get the entry field of the header
    pub fn entry(&self) -> usize {
        self.header.e_entry as usize
    }

    /// Test function to run. Remove and replace with proper syscall
    pub fn run(self) {
        let mut active_table = unsafe { ActivePageTable::new() };

        for segment in self.segments() {
            println!("Segment {:X} flags {:X} off {:X} virt {:X} phys {:X} file {} mem {} align {}",
                        segment.p_type, segment.p_flags, segment.p_offset,
                        segment.p_vaddr, segment.p_paddr, segment.p_filesz,
                        segment.p_memsz, segment.p_align);

            if segment.p_type == program_header::PT_LOAD {
                let start_page = Page::containing_address(VirtualAddress::new(segment.p_vaddr as usize));
                let end_page = Page::containing_address(VirtualAddress::new((segment.p_vaddr + segment.p_memsz) as usize));

                for page in Page::range_inclusive(start_page, end_page) {
                    active_table.map(page, entry::NO_EXECUTE | entry::WRITABLE);
                }

                unsafe {
                    // Update the page table
                    tlb::flush_all();

                    // Copy file data
                    memcpy(segment.p_vaddr as *mut u8,
                            (self.data.as_ptr() as usize + segment.p_offset as usize) as *const u8,
                            segment.p_filesz as usize);
                    // Set BSS
                    memset((segment.p_vaddr + segment.p_filesz) as *mut u8,
                            0,
                            (segment.p_memsz - segment.p_filesz) as usize);
                }

                let mut flags = entry::NO_EXECUTE | entry::USER_ACCESSIBLE;

                if segment.p_flags & program_header::PF_R == program_header::PF_R {
                    flags.insert(entry::PRESENT);
                }

                // W ^ X. If it is executable, do not allow it to be writable, even if requested
                if segment.p_flags & program_header::PF_X == program_header::PF_X {
                    flags.remove(entry::NO_EXECUTE);
                } else if segment.p_flags & program_header::PF_W == program_header::PF_W {
                    flags.insert(entry::WRITABLE);
                }

                for page in Page::range_inclusive(start_page, end_page) {
                    println!("{:X}: {:?}", page.start_address().get(), flags);
                    active_table.remap(page, flags);
                }

                unsafe {
                    // Update the page table
                    tlb::flush_all();
                }
            }
        }

        unsafe {
            // Map stack
            let start_page = Page::containing_address(VirtualAddress::new(0x80000000));
            let end_page = Page::containing_address(VirtualAddress::new(0x80000000 + 64*1024 - 1));

            for page in Page::range_inclusive(start_page, end_page) {
                active_table.map(page, entry::NO_EXECUTE | entry::WRITABLE | entry::USER_ACCESSIBLE);
            }

            // Update the page table
            tlb::flush_all();

            // Clear stack
            memset(0x80000000 as *mut u8, 0, 64 * 1024);

            // Go to usermode
            usermode(self.entry(), 0x80000000 + 64*1024 - 256);
        }
    }
}

pub struct ElfSegments<'a> {
    data: &'a [u8],
    header: &'a header::Header,
    i: usize
}

impl<'a> Iterator for ElfSegments<'a> {
    type Item = &'a program_header::ProgramHeader;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.header.e_phnum as usize {
            let item = unsafe {
                &* ((
                        self.data.as_ptr() as usize
                        + self.header.e_phoff as usize
                        + self.i * self.header.e_phentsize as usize
                    ) as *const program_header::ProgramHeader)
            };
            self.i += 1;
            Some(item)
        } else {
            None
        }
    }
}
