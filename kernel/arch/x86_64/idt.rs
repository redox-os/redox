#[repr(packed)]
pub struct IdtDescriptor {
    pub size: u16,
    pub ptr: u64
}

#[derive(Debug)]
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
