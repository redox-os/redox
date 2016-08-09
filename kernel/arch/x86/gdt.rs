#[repr(packed)]
pub struct GdtDescriptor {
    pub size: u16,
    pub ptr: u32
}

#[repr(packed)]
pub struct GdtEntry {
    pub limitl: u16,
    pub basel: u16,
    pub basem: u8,
    pub attribute: u8,
    pub flags_limith: u8,
    pub baseh: u8
}
