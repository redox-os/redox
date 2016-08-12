//! ELF executables

use collections::{String, Vec};

use core::{mem, ptr, str, slice};

use common::slice::GetSlice;

pub use self::arch::*;

#[cfg(target_arch = "x86")]
#[path="x86/elf.rs"]
mod arch;

#[cfg(target_arch = "x86_64")]
#[path="x86_64/elf.rs"]
mod arch;

/// An ELF executable
pub struct Elf<'a> {
    pub data: &'a [u8],
}

impl<'a> Elf<'a> {
    /// Create a ELF executable from data
    pub fn from(data: &'a [u8]) -> Result<Elf<'a>, String> {
        if data.len() < mem::size_of::<ElfHeader>() {
            Err(format!("Elf: Not enough data: {} < {}", data.len(), mem::size_of::<ElfHeader>()))
        } else if data.get_slice(..4) != b"\x7FELF" {
            Err(format!("Elf: Invalid magic: {:?} != {:?}", data.get_slice(..4), b"\x7FELF"))
        } else if data.get(4) != Some(&ELF_CLASS) {
            Err(format!("Elf: Invalid architecture: {:?} != {:?}", data.get(4), Some(&ELF_CLASS)))
        } else {
            Ok(Elf { data: data })
        }
    }

    pub unsafe fn load_segments(&self) -> Vec<ElfSegment> {
        let mut segments = Vec::new();

        let header = &*(self.data.as_ptr() as usize as *const ElfHeader);

        for i in 0..header.ph_len {
            let segment = ptr::read((self.data.as_ptr() as usize + header.ph_off as usize + i as usize * header.ph_ent_len as usize) as *const ElfSegment);

            if segment._type == 1 || segment._type == 7 {
                segments.push(segment);
            }
        }

        segments
    }

    /// Get the entry field of the header
    pub unsafe fn entry(&self) -> usize {
        let header = &*(self.data.as_ptr() as usize as *const ElfHeader);
        header.entry as usize
    }

    /// ELF symbol
    pub unsafe fn symbol(&self, name: &str) -> usize {
        let header = &*(self.data.as_ptr() as usize as *const ElfHeader);

        let sh_str_section =
            &*((self.data.as_ptr() as usize + header.sh_off as usize +
                header.sh_str_index as usize *
                header.sh_ent_len as usize) as *const ElfSection);

        let mut sym_section = &*((self.data.as_ptr() as usize + header.sh_off as usize) as *const ElfSection);

        let mut str_section = &*((self.data.as_ptr() as usize + header.sh_off as usize) as *const ElfSection);

        for i in 0..header.sh_len {
            let section =
                &*((self.data.as_ptr() as usize + header.sh_off as usize +
                    i as usize *
                    header.sh_ent_len as usize) as *const ElfSection);

            let section_name_ptr =
                (self.data.as_ptr() as usize + sh_str_section.off as usize + section.name as usize) as *const u8;
            let mut section_name_len = 0;
            for j in 0..4096 {
                section_name_len = j;
                if ptr::read(section_name_ptr.offset(j)) == 0 {
                    break;
                }
            }
            let section_name =
                str::from_utf8_unchecked(slice::from_raw_parts(section_name_ptr,
                                                               section_name_len as usize));

            if section_name == ".symtab" {
                sym_section = section;
            } else if section_name == ".strtab" {
                str_section = section;
            }
        }

        if sym_section.off > 0 && str_section.off > 0 {
            if sym_section.ent_len > 0 {
                let len = sym_section.len / sym_section.ent_len;
                for i in 0..len {
                    let symbol = &*((self.data.as_ptr() as usize + sym_section.off as usize + i as usize * sym_section.ent_len as usize) as *const ElfSymbol);

                    let symbol_name_ptr =
                        (self.data.as_ptr() as usize + str_section.off as usize +
                         symbol.name as usize) as *const u8;
                    let mut symbol_name_len = 0;
                    for j in 0..4096 {
                        symbol_name_len = j;
                        if ptr::read(symbol_name_ptr.offset(j)) == 0 {
                            break;
                        }
                    }
                    let symbol_name = str::from_utf8_unchecked(slice::from_raw_parts(symbol_name_ptr, symbol_name_len as usize));

                    if name == symbol_name {
                        return symbol.value as usize;
                    }
                }
            } else {
                debug!("No sym_section ent len\n");
            }
        } else {
            debug!("No sym_section or str_section\n");
        }

        0
    }
}
