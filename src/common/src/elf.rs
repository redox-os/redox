//! ELF executables

use core::ops::Drop;
use core::ptr;

use common::debug::*;
use common::memory;
use common::string::*;

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
               *((file_data + 3) as *const u8) == 'F' as u8 &&
               *((file_data + 4) as *const u8) == 1 {
                let size = memory::alloc_size(file_data);
                data = memory::alloc(size);
                ptr::copy(file_data as *const u8, data as *mut u8, size);
            } else {
                d("Invalid ELF Format\n");
                data = 0;
            }
        }

        ELF { data: data }
    }

    /// Debug
    pub unsafe fn d(&self) {
        if self.data > 0 {
            d("Debug ELF\n");
            let header = &*(self.data as *const ELFHeader);

            d("Magic: ");
            for i in 0..4 {
                dbh(header.magic[i]);
            }
            dl();

            d("Class: ");
            dbh(header.class);
            dl();

            d("Endian: ");
            dbh(header.endian);
            dl();

            d("Version: ");
            dbh(header.ver);
            dl();

            d("ABI: ");
            for i in 0..2 {
                dbh(header.abi[i]);
            }
            dl();

            d("Type: ");
            dh(header._type as usize);
            dl();

            d("Machine: ");
            dh(header.machine as usize);
            dl();

            d("Version 2: ");
            dh(header.ver_2 as usize);
            dl();

            d("Entry: ");
            dh(header.entry as usize);
            dl();

            d("Program Header Table: ");
            dh(header.ph_off as usize);
            d(" ent_len: ");
            dd(header.ph_ent_len as usize);
            d(" len: ");
            dd(header.ph_len as usize);
            dl();

            d("Section Header Table: ");
            dh(header.sh_off as usize);
            d(" ent_len: ");
            dd(header.sh_ent_len as usize);
            d(" len: ");
            dd(header.sh_len as usize);
            dl();

            d("Flags: ");
            dh(header.flags as usize);
            dl();

            d("Section Header Strings: ");
            dd(header.sh_str_index as usize);
            dl();

            let sh_str_section = &*((self.data + header.sh_off as usize + header.sh_str_index as usize * header.sh_ent_len as usize) as *const ELFSection);

            let mut sym_section = &*((self.data + header.sh_off as usize) as *const ELFSection);

            let mut str_section = &*((self.data + header.sh_off as usize) as *const ELFSection);

            d("Section Headers:");
            dl();

            for i in 0..header.sh_len {
                let section = &*((self.data + header.sh_off as usize + i as usize * header.sh_ent_len as usize) as *const ELFSection);

                let name = String::from_c_str((self.data + sh_str_section.off as usize + section.name as usize) as *const u8);

                if name == ".symtab" {
                    sym_section = section;
                } else if name == ".strtab" {
                    str_section = section;
                }

                d("    Section ");
                dd(i as usize);
                d(": ");
                name.d();
                dl();

                d("    Type: ");
                dh(section._type as usize);
                dl();

                d("    Flags: ");
                dh(section.flags as usize);
                dl();

                d("    Addr: ");
                dh(section.addr as usize);
                dl();

                d("    Offset: ");
                dh(section.off as usize);
                dl();

                d("    Length: ");
                dd(section.len as usize);
                dl();

                d("    Link: ");
                dd(section.link as usize);
                dl();

                d("    Info: ");
                dd(section.info as usize);
                dl();

                d("    Address Align: ");
                dd(section.addr_align as usize);
                dl();

                d("    Entry Length: ");
                dd(section.ent_len as usize);
                dl();

                dl();
            }

            if sym_section.off > 0 && str_section.off > 0 {
                if sym_section.ent_len > 0 {
                    let len = sym_section.len / sym_section.ent_len;
                    d("Symbols: ");
                    dd(len as usize);
                    dl();
                    for i in 0..len {
                        let symbol = &*((self.data + sym_section.off as usize + i as usize * sym_section.ent_len as usize) as *const ELFSymbol);

                        let name = String::from_c_str((self.data + str_section.off as usize + symbol.name as usize) as *const u8);

                        d("    Symbol ");
                        dd(i as usize);
                        d(": ");
                        name.d();
                        dl();

                        d("    Value: ");
                        dh(symbol.value as usize);
                        dl();

                        d("    Size: ");
                        dd(symbol.size as usize);
                        dl();

                        d("    Info: ");
                        dbh(symbol.info);
                        dl();

                        d("    Other: ");
                        dbh(symbol.other);
                        dl();

                        d("    Section: ");
                        dd(symbol.sh_index as usize);
                        dl();

                        dl();
                    }
                } else {
                    d("Symbol length is 0\n");
                }
            } else {
                d("No symbol section or string section");
            }

            dl();
        } else {
            d("Empty ELF File\n");
        }
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

                let section_name = String::from_c_str((self.data + sh_str_section.off as usize + section.name as usize) as *const u8);

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

                        if name == String::from_c_str((self.data + str_section.off as usize + symbol.name as usize) as *const u8) {
                            return symbol.value as usize;
                        }
                    }
                }
            }

            dl();
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
