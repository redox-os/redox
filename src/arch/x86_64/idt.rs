use core::mem;

pub static mut IDTR: IdtDescriptor = IdtDescriptor {
    size: 0,
    offset: 0
};

pub static mut IDT: [IdtEntry; 256] = [IdtEntry::new(); 256];

#[repr(packed)]
pub struct IdtDescriptor {
    pub size: u16,
    pub offset: u64
}

impl IdtDescriptor {
    pub fn set_slice(&mut self, slice: &'static [IdtEntry]) {
        self.size = (slice.len() * mem::size_of::<IdtEntry>() - 1) as u16;
        self.offset = slice.as_ptr() as u64;
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct IdtEntry {
    pub offsetl: u16,
    pub selector: u16,
    pub zero: u8,
    pub attribute: u8,
    pub offsetm: u16,
    pub offseth: u32,
    pub zero2: u32
}

impl IdtEntry {
    pub const fn new() -> IdtEntry {
        IdtEntry {
            offsetl: 0,
            selector: 0,
            zero: 0,
            attribute: 0,
            offsetm: 0,
            offseth: 0,
            zero2: 0
        }
    }

    pub fn set_offset(&mut self, base: usize) {
        self.offsetl = base as u16;
        self.offsetm = (base >> 16) as u16;
        self.offseth = (base >> 32) as u32;
    }
}
