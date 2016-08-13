pub const GDT_NULL: usize = 0;
pub const GDT_KERNEL_CODE: usize = 1;
pub const GDT_KERNEL_DATA: usize = 2;
pub const GDT_USER_CODE: usize = 3;
pub const GDT_USER_DATA: usize = 4;
pub const GDT_USER_TLS: usize = 5;
pub const GDT_TSS: usize = 6;

#[repr(packed)]
pub struct GdtDescriptor {
    pub size: u16,
    pub ptr: u64
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

impl GdtEntry {
    pub fn set_base(&mut self, base: usize) {
        self.basel = base as u16;
        self.basem = (base >> 16) as u8;
        self.baseh = (base >> 24) as u8;
    }
}
