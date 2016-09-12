//! ELF executables

use collections::String;

use core::str;

#[cfg(target_arch = "x86")]
use goblin::elf32::{header, program_header};

#[cfg(target_arch = "x86_64")]
use goblin::elf64::{header, program_header};

use arch::externs::{memcpy, memset};
use arch::paging::{entry, VirtualAddress};
use arch::start::usermode;
use context;
use syscall::{Error, Result as SysResult};

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
    pub fn run(self) -> SysResult<!> {
        let stack_addr = 0x80000000;
        let stack_size = 64 * 1024;
        {
            let contexts = context::contexts();
            let context_lock = contexts.current().ok_or(Error::NoProcess)?;
            let mut context = context_lock.write();

            // Unmap previous image and stack
            context.image.clear();
            context.stack.take();

            for segment in self.segments() {
                if segment.p_type == program_header::PT_LOAD {
                    let mut memory = context::memory::Memory::new(
                        VirtualAddress::new(segment.p_vaddr as usize),
                        segment.p_memsz as usize,
                        entry::NO_EXECUTE | entry::WRITABLE
                    );

                    unsafe {
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

                    memory.remap(flags, true);

                    context.image.push(memory);
                }
            }

            // Map stack
            context.stack = Some(context::memory::Memory::new(
                VirtualAddress::new(stack_addr),
                stack_size,
                entry::NO_EXECUTE | entry::WRITABLE | entry::USER_ACCESSIBLE
            ));

            // Clear stack
            unsafe { memset(stack_addr as *mut u8, 0, stack_size); }
        }

        // Go to usermode
        unsafe { usermode(self.entry(), stack_addr + stack_size - 256); }
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
