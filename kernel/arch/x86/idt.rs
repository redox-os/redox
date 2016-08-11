#[repr(packed)]
pub struct IdtDescriptor {
    pub size: u16,
    pub ptr: u32
}

#[repr(packed)]
pub struct IdtEntry {
    pub offsetl: u16,
    pub selector: u16,
    pub zero: u8,
    pub attribute: u8,
    pub offseth: u16
}
