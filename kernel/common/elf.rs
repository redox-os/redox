//! ELF executables

use core::{ptr, str, slice};

use common::{debug, memory};

#[cfg(target_arch = "x86")]
pub const ELF_OFFSET: usize = 0x1000;

#[cfg(target_arch = "x86_64")]
pub const ELF_OFFSET: usize = 0x200000;

/// An ELF header
#[repr(packed)]
pub struct ELFHeader {
    /// The "magic number" (4 bytes)
    pub magic: [u8; 4],
    /// 64 or 32 bit?
    pub class: u8,
    /// Little (1) or big endianness (2)?
    pub endian: u8,
    /// The ELF version (set to 1 for default)
    pub ver: u8,
    /// Operating system ABI (0x03 for Linux)
    pub abi: [u8; 2],
    /// Unused
    pub pad: [u8; 7],
    /// Specify whether the object is relocatable, executable, shared, or core (in order).
    pub _type: u16,
    /// Instruction set archcitecture
    pub machine: u16,
    /// Second version
    pub ver_2: u32,
    /// The ELF entry
    pub entry: u32,
    /// The program header table offset
    pub ph_off: u32,
    /// The section header table offset
    pub sh_off: u32,
    /// The flags set
    pub flags: u32,
    /// The header table length
    pub h_len: u16,
    /// The program header table entry length
    pub ph_ent_len: u16,
    /// The program head table length
    pub ph_len: u16,
    /// The section header table entry length
    pub sh_ent_len: u16,
    /// The section header table length
    pub sh_len: u16,
    /// The section header table string index
    pub sh_str_index: u16,
}

/// An ELF segment
#[repr(packed)]
pub struct ELFSegment {
    pub _type: u32,
    pub off: u32,
    pub vaddr: u32,
    pub paddr: u32,
    pub file_len: u32,
    pub mem_len: u32,
    pub flags: u32,
    pub align: u32,
}

/// An ELF section
#[repr(packed)]
pub struct ELFSection {
    pub name: u32,
    pub _type: u32,
    pub flags: u32,
    pub addr: u32,
    pub off: u32,
    pub len: u32,
    pub link: u32,
    pub info: u32,
    pub addr_align: u32,
    pub ent_len: u32,
}

/// An ELF symbol
#[repr(packed)]
pub struct ELFSymbol {
    pub name: u32,
    pub value: u32,
    pub size: u32,
    pub info: u8,
    pub other: u8,
    pub sh_index: u16,
}

/// An ELF executable
pub struct ELF {
    pub data: usize,
}

impl ELF {
    /// Create a new empty ELF executable
    pub fn new() -> Self {
        ELF { data: 0 }
    }

    /// Create a ELF executable from data
    pub fn from_data(file_data: usize) -> Self {
        let data;
        unsafe {
            if file_data > 0 && *(file_data as *const u8) == 0x7F &&
               *((file_data + 1) as *const u8) == 'E' as u8 &&
               *((file_data + 2) as *const u8) == 'L' as u8 &&
               *((file_data + 3) as *const u8) == 'F' as u8 {
                let size = memory::alloc_size(file_data);
                data = memory::alloc(size);
                ptr::copy(file_data as *const u8, data as *mut u8, size);
            } else {
                debug::d("Invalid ELF Format\n");
                data = 0;
            }
        }

        ELF { data: data }
    }

    /// Debug
    pub unsafe fn d(&self) {
        if self.data > 0 {
            debug::d("Debug ELF\n");
            let header = &*(self.data as *const ELFHeader);

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

            let sh_str_section = &*((self.data + header.sh_off as usize + header.sh_str_index as usize * header.sh_ent_len as usize) as *const ELFSection);

            let mut sym_section = &*((self.data + header.sh_off as usize) as *const ELFSection);

            let mut str_section = &*((self.data + header.sh_off as usize) as *const ELFSection);

            debug::d("Program Headers:");
            debug::dl();

            for i in 0..header.ph_len {
                let segment = &*((self.data + header.ph_off as usize + i as usize * header.ph_ent_len as usize) as *const ELFSegment);

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
                let section = &*((self.data + header.sh_off as usize + i as usize * header.sh_ent_len as usize) as *const ELFSection);

                let section_name_ptr = (self.data + sh_str_section.off as usize + section.name as usize) as *const u8;
                let mut section_name_len = 0;
                for j in 0..4096 {
                    section_name_len = j;
                    if ptr::read(section_name_ptr.offset(j)) == 0 {
                        break;
                    }
                }
                let section_name = str::from_utf8_unchecked(slice::from_raw_parts(section_name_ptr, section_name_len as usize));

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
                        let symbol = &*((self.data + sym_section.off as usize + i as usize * sym_section.ent_len as usize) as *const ELFSymbol);

                        let symbol_name_ptr = (self.data + str_section.off as usize + symbol.name as usize) as *const u8;
                        let mut symbol_name_len = 0;
                        for j in 0..4096 {
                            symbol_name_len = j;
                            if ptr::read(symbol_name_ptr.offset(j)) == 0 {
                                break;
                            }
                        }
                        let symbol_name = str::from_utf8_unchecked(slice::from_raw_parts(symbol_name_ptr, symbol_name_len as usize));

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
        } else {
            debug::d("Empty ELF File\n");
        }
    }

    pub unsafe fn load_segment(&self) -> Option<ELFSegment> {
        if self.data > 0 {
            let header = &*(self.data as *const ELFHeader);

            for i in 0..header.ph_len {
                let segment = ptr::read((self.data + header.ph_off as usize + i as usize * header.ph_ent_len as usize) as *const ELFSegment);

                if segment._type == 1 {
                    return Some(segment);
                }
            }
        }

        None
    }

    /// Get the entry field of the header
    pub unsafe fn entry(&self) -> usize {
        if self.data > 0 {
            // TODO: Support 64-bit version
            // TODO: Get information from program headers
            let header = &*(self.data as *const ELFHeader);
            return header.entry as usize;
        }

        0
    }

    /// ELF symbol
    pub unsafe fn symbol(&self, name: &str) -> usize {
        if self.data > 0 {
            let header = &*(self.data as *const ELFHeader);

            let sh_str_section = &*((self.data + header.sh_off as usize + header.sh_str_index as usize * header.sh_ent_len as usize) as *const ELFSection);

            let mut sym_section = &*((self.data + header.sh_off as usize) as *const ELFSection);

            let mut str_section = &*((self.data + header.sh_off as usize) as *const ELFSection);

            for i in 0..header.sh_len {
                let section = &*((self.data + header.sh_off as usize + i as usize * header.sh_ent_len as usize) as *const ELFSection);

                let section_name_ptr = (self.data + sh_str_section.off as usize + section.name as usize) as *const u8;
                let mut section_name_len = 0;
                for j in 0..4096 {
                    section_name_len = j;
                    if ptr::read(section_name_ptr.offset(j)) == 0 {
                        break;
                    }
                }
                let section_name = str::from_utf8_unchecked(slice::from_raw_parts(section_name_ptr, section_name_len as usize));

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
                        let symbol = &*((self.data + sym_section.off as usize + i as usize * sym_section.ent_len as usize) as *const ELFSymbol);

                        let symbol_name_ptr = (self.data + str_section.off as usize + symbol.name as usize) as *const u8;
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
                    debug::d("No sym_section ent len\n");
                }
            } else {
                debug::d("No sym_section or str_section\n");
            }
        } else {
            debug::d("No data\n");
        }

        0
    }
}

impl Drop for ELF {
    fn drop(&mut self) {
        unsafe {
            if self.data > 0 {
                memory::unalloc(self.data);
                self.data = 0;
            }
        }
    }
}
