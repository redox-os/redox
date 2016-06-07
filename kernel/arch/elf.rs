//! ELF executables

use collections::{String, Vec};

use core::{mem, ptr, slice, str};

use common::debug;
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
            Err(format!("Elf: Not enough data: {} < {}",
                        data.len(),
                        mem::size_of::<ElfHeader>()))
        } else if data.get_slice(..4) != b"\x7FELF" {
            Err(format!("Elf: Invalid magic: {:?} != {:?}",
                        data.get_slice(..4),
                        b"\x7FELF"))
        } else if data.get(4) != Some(&ELF_CLASS) {
            Err(format!("Elf: Invalid architecture: {:?} != {:?}",
                        data.get(4),
                        Some(&ELF_CLASS)))
        } else {
            Ok(Elf { data: data })
        }
    }

    /// Debug
    pub unsafe fn d(&self) {
        debug::d("Debug ELF\n");
        let header = &*(self.data.as_ptr() as *const ElfHeader);

        debug::d("Magic: ");
        for i in 0..4 {
            debug::dbh(header.magic[i]);
        }
        debug::dl();

        debug::d("Class: ");
        debug::dbh(header.class);
        debug::dl();

        debug::d("Endian: ");
        debug::dbh(header.endian);
        debug::dl();

        debug::d("Version: ");
        debug::dbh(header.ver);
        debug::dl();

        debug::d("ABI: ");
        for i in 0..2 {
            debug::dbh(header.abi[i]);
        }
        debug::dl();

        debug::d("Type: ");
        debug::dh(header._type as usize);
        debug::dl();

        debug::d("Machine: ");
        debug::dh(header.machine as usize);
        debug::dl();

        debug::d("Version 2: ");
        debug::dh(header.ver_2 as usize);
        debug::dl();

        debug::d("Entry: ");
        debug::dh(header.entry as usize);
        debug::dl();

        debug::d("Program Header Table: ");
        debug::dh(header.ph_off as usize);
        debug::d(" ent_len: ");
        debug::dd(header.ph_ent_len as usize);
        debug::d(" len: ");
        debug::dd(header.ph_len as usize);
        debug::dl();

        debug::d("Section Header Table: ");
        debug::dh(header.sh_off as usize);
        debug::d(" ent_len: ");
        debug::dd(header.sh_ent_len as usize);
        debug::d(" len: ");
        debug::dd(header.sh_len as usize);
        debug::dl();

        debug::d("Flags: ");
        debug::dh(header.flags as usize);
        debug::dl();

        debug::d("Section Header Strings: ");
        debug::dd(header.sh_str_index as usize);
        debug::dl();

        let sh_str_section =
            &*((self.data.as_ptr() as usize + header.sh_off as usize +
                header.sh_str_index as usize *
                header.sh_ent_len as usize) as *const ElfSection);

        let mut sym_section =
            &*((self.data.as_ptr() as usize + header.sh_off as usize) as *const ElfSection);

        let mut str_section =
            &*((self.data.as_ptr() as usize + header.sh_off as usize) as *const ElfSection);

        debug::d("Program Headers:");
        debug::dl();

        for i in 0..header.ph_len {
            let segment =
                &*((self.data.as_ptr() as usize + header.ph_off as usize +
                    i as usize *
                    header.ph_ent_len as usize) as *const ElfSegment);

            debug::d("    Section ");
            debug::dd(i as usize);
            debug::dl();

            debug::d("    Type: ");
            debug::dh(segment._type as usize);
            debug::dl();

            debug::d("    Offset: ");
            debug::dh(segment.off as usize);
            debug::dl();

            debug::d("    VAddr: ");
            debug::dh(segment.vaddr as usize);
            debug::dl();

            debug::d("    PAddr: ");
            debug::dh(segment.paddr as usize);
            debug::dl();

            debug::d("    File Length: ");
            debug::dd(segment.file_len as usize);
            debug::dl();

            debug::d("    Mem Length: ");
            debug::dd(segment.mem_len as usize);
            debug::dl();

            debug::d("    Flags: ");
            debug::dh(segment.flags as usize);
            debug::dl();

            debug::d("    Align: ");
            debug::dd(segment.align as usize);
            debug::dl();

            debug::dl();
        }

        debug::d("Section Headers:");
        debug::dl();

        for i in 0..header.sh_len {
            let section =
                &*((self.data.as_ptr() as usize + header.sh_off as usize +
                    i as usize *
                    header.sh_ent_len as usize) as *const ElfSection);

            let section_name_ptr =
                (self.data.as_ptr() as usize + sh_str_section.off as usize +
                 section.name as usize) as *const u8;
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

            debug::d("    Section ");
            debug::dd(i as usize);
            debug::d(": ");
            debug::d(section_name);
            debug::dl();

            debug::d("    Type: ");
            debug::dh(section._type as usize);
            debug::dl();

            debug::d("    Flags: ");
            debug::dh(section.flags as usize);
            debug::dl();

            debug::d("    Addr: ");
            debug::dh(section.addr as usize);
            debug::dl();

            debug::d("    Offset: ");
            debug::dh(section.off as usize);
            debug::dl();

            debug::d("    Length: ");
            debug::dd(section.len as usize);
            debug::dl();

            debug::d("    Link: ");
            debug::dd(section.link as usize);
            debug::dl();

            debug::d("    Info: ");
            debug::dd(section.info as usize);
            debug::dl();

            debug::d("    Address Align: ");
            debug::dd(section.addr_align as usize);
            debug::dl();

            debug::d("    Entry Length: ");
            debug::dd(section.ent_len as usize);
            debug::dl();

            debug::dl();
        }

        if sym_section.off > 0 && str_section.off > 0 {
            if sym_section.ent_len > 0 {
                let len = sym_section.len / sym_section.ent_len;
                debug::d("Symbols: ");
                debug::dd(len as usize);
                debug::dl();
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
                    let symbol_name =
                        str::from_utf8_unchecked(slice::from_raw_parts(symbol_name_ptr,
                                                                       symbol_name_len as usize));

                    debug::d("    Symbol ");
                    debug::dd(i as usize);
                    debug::d(": ");
                    debug::d(symbol_name);

                    debug::d(" Value: ");
                    debug::dh(symbol.value as usize);

                    debug::d(" Size: ");
                    debug::dd(symbol.size as usize);

                    debug::d(" Info: ");
                    debug::dbh(symbol.info);

                    debug::d(" Other: ");
                    debug::dbh(symbol.other);

                    debug::d(" Section: ");
                    debug::dd(symbol.sh_index as usize);
                    debug::dl();
                }
            } else {
                debug::d("Symbol length is 0\n");
            }
        } else {
            debug::d("No symbol section or string section");
        }

        debug::dl();
    }

    pub unsafe fn load_segment(&self) -> Vec<ElfSegment> {
        let mut segments = Vec::new();

        let header = &*(self.data.as_ptr() as usize as *const ElfHeader);

        for i in 0..header.ph_len {
            let segment = ptr::read((self.data.as_ptr() as usize + header.ph_off as usize + i as usize * header.ph_ent_len as usize) as *const ElfSegment);

            if segment._type == 1 {
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

        let mut sym_section =
            &*((self.data.as_ptr() as usize + header.sh_off as usize) as *const ElfSection);

        let mut str_section =
            &*((self.data.as_ptr() as usize + header.sh_off as usize) as *const ElfSection);

        for i in 0..header.sh_len {
            let section =
                &*((self.data.as_ptr() as usize + header.sh_off as usize +
                    i as usize *
                    header.sh_ent_len as usize) as *const ElfSection);

            let section_name_ptr =
                (self.data.as_ptr() as usize + sh_str_section.off as usize +
                 section.name as usize) as *const u8;
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
                    let symbol_name =
                        str::from_utf8_unchecked(slice::from_raw_parts(symbol_name_ptr,
                                                                       symbol_name_len as usize));

                    if name == symbol_name {
                        return symbol.value as usize;
                    }
                }
            } else {
                debug::d("No sym_section ent len\n");
            }
        } else {
            debug::d("No sym_section or str_section\n");
        }

        0
    }
}
