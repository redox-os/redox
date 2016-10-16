//! ELF executables

use collections::{String, Vec};

use core::{ptr, str};

use common::slice::GetSlice;

#[cfg(target_arch = "x86")]
use goblin::elf32::{header, program_header};

#[cfg(target_arch = "x86_64")]
use goblin::elf64::{header, program_header};

/// An ELF executable
pub struct Elf<'a> {
    pub data: &'a [u8],
}

impl<'a> Elf<'a> {
    /// Create a ELF executable from data
    pub fn from(data: &'a [u8]) -> Result<Elf<'a>, String> {
        // this verification code is a use case for `goblin::elf::header::peek`
        // but you 1. might want error codes in your control
        // 2. might not want binary pre-check validation controlled by an upstream crate
        if data.len() < header::SIZEOF_EHDR {
            Err(format!("Elf: Not enough data: {} < {}", data.len(), header::SIZEOF_EHDR))
        } else if data.get_slice(..header::SELFMAG) != header::ELFMAG {
            Err(format!("Elf: Invalid magic: {:?} != {:?}", data.get_slice(..header::SELFMAG), header::ELFMAG))
        } else if data.get(header::EI_CLASS) != Some(&header::ELFCLASS) {
            Err(format!("Elf: Invalid architecture: {:?} != {:?}", data.get(header::EI_CLASS), header::ELFCLASS))
        } else {
            Ok(Elf { data: data })
        }
    }

    pub unsafe fn load_segments(&self) -> Vec<program_header::ProgramHeader> {
        let mut segments = Vec::new();

        let header = &*(self.data.as_ptr() as usize as *const header::Header);

        for i in 0..header.e_phnum {
            let segment = ptr::read((self.data.as_ptr() as usize + header.e_phoff as usize + i as usize * header.e_phentsize as usize) as *const program_header::ProgramHeader);

            if segment.p_type == program_header::PT_LOAD || segment.p_type == program_header::PT_TLS {
                segments.push(segment);
            }
        }

        segments
    }

    /// Get the entry field of the header
    pub unsafe fn entry(&self) -> usize {
        let header = &*(self.data.as_ptr() as usize as *const header::Header);
        header.e_entry as usize
    }
}
