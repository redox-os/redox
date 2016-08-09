pub const ELF_CLASS: u8 = 1;
pub type ElfAddr = u32;
pub type ElfHalf = u16;
pub type ElfOff = u32;
pub type ElfWord = u32;

/// An ELF header
#[repr(packed)]
#[derive(Debug)]
pub struct ElfHeader {
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
    pub _type: ElfHalf,
    /// Instruction set archcitecture
    pub machine: ElfHalf,
    /// Second version
    pub ver_2: ElfWord,
    /// The ELF entry
    pub entry: ElfAddr,
    /// The program header table offset
    pub ph_off: ElfOff,
    /// The section header table offset
    pub sh_off: ElfOff,
    /// The flags set
    pub flags: ElfWord,
    /// The header table length
    pub h_len: ElfHalf,
    /// The program header table entry length
    pub ph_ent_len: ElfHalf,
    /// The program head table length
    pub ph_len: ElfHalf,
    /// The section header table entry length
    pub sh_ent_len: ElfHalf,
    /// The section header table length
    pub sh_len: ElfHalf,
    /// The section header table string index
    pub sh_str_index: ElfHalf,
}

/// An ELF segment
#[repr(packed)]
#[derive(Debug)]
pub struct ElfSegment {
    pub _type: ElfWord,
    pub off: ElfOff,
    pub vaddr: ElfAddr,
    pub paddr: ElfAddr,
    pub file_len: ElfWord,
    pub mem_len: ElfWord,
    pub flags: ElfWord,
    pub align: ElfWord,
}

/// An ELF section
#[repr(packed)]
#[derive(Debug)]
pub struct ElfSection {
    pub name: ElfWord,
    pub _type: ElfWord,
    pub flags: ElfWord,
    pub addr: ElfAddr,
    pub off: ElfOff,
    pub len: ElfWord,
    pub link: ElfWord,
    pub info: ElfWord,
    pub addr_align: ElfWord,
    pub ent_len: ElfWord,
}

/// An ELF symbol
#[repr(packed)]
#[derive(Debug)]
pub struct ElfSymbol {
    pub name: ElfWord,
    pub value: ElfAddr,
    pub size: ElfWord,
    pub info: u8,
    pub other: u8,
    pub sh_index: ElfHalf,
}
