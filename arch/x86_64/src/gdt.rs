//! Global descriptor table

use core::mem;

pub const GDT_NULL: usize = 0;
pub const GDT_KERNEL_CODE: usize = 1;
pub const GDT_KERNEL_DATA: usize = 2;
pub const GDT_USER_CODE: usize = 3;
pub const GDT_USER_DATA: usize = 4;
pub const GDT_USER_TLS: usize = 5;
pub const GDT_TSS: usize = 6;

pub static mut GDTR: GdtDescriptor = GdtDescriptor {
    size: 0,
    offset: 0
};

pub static mut GDT: [GdtEntry; 5] = [GdtEntry::new(); 5];

pub unsafe fn init() {
    GDT[GDT_KERNEL_CODE].set_access(GDT_PRESENT | GDT_RING_0 | GDT_SYSTEM | GDT_EXECUTABLE | GDT_PRIVILEGE);
    GDT[GDT_KERNEL_CODE].set_flags(GDT_LONG_MODE);

    GDT[GDT_KERNEL_DATA].set_access(GDT_PRESENT | GDT_RING_0 | GDT_SYSTEM | GDT_PRIVILEGE);
    GDT[GDT_KERNEL_DATA].set_flags(GDT_LONG_MODE);

    GDT[GDT_USER_CODE].set_access(GDT_PRESENT | GDT_RING_3 | GDT_SYSTEM | GDT_EXECUTABLE | GDT_PRIVILEGE);
    GDT[GDT_USER_CODE].set_flags(GDT_LONG_MODE);

    GDT[GDT_USER_DATA].set_access(GDT_PRESENT | GDT_RING_3 | GDT_SYSTEM | GDT_PRIVILEGE);
    GDT[GDT_USER_DATA].set_flags(GDT_LONG_MODE);

    GDTR.set_slice(&GDT);

    init_ap();
}

pub unsafe fn init_ap() {
    GDTR.load();
}

bitflags! {
    pub flags GdtAccess: u8 {
        const GDT_PRESENT = 1 << 7,
        const GDT_RING_0 = 0 << 5,
        const GDT_RING_1 = 1 << 5,
        const GDT_RING_2 = 2 << 5,
        const GDT_RING_3 = 3 << 5,
        const GDT_SYSTEM = 1 << 4,
        const GDT_EXECUTABLE = 1 << 3,
        const GDT_CONFORMING = 1 << 2,
        const GDT_PRIVILEGE = 1 << 1,
        const GDT_DIRTY = 1,
    }
}

bitflags! {
    pub flags GdtFlags: u8 {
        const GDT_PAGE_SIZE = 1 << 7,
        const GDT_PROTECTED_MODE = 1 << 6,
        const GDT_LONG_MODE = 1 << 5
    }
}

#[repr(packed)]
pub struct GdtDescriptor {
    pub size: u16,
    pub offset: u64
}

impl GdtDescriptor {
    pub fn set_slice(&mut self, slice: &'static [GdtEntry]) {
        self.size = (slice.len() * mem::size_of::<GdtEntry>() - 1) as u16;
        self.offset = slice.as_ptr() as u64;
    }

    pub unsafe fn load(&self) {
        asm!("lgdt [rax]" : : "{rax}"(self as *const _ as usize) : : "intel", "volatile");
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct GdtEntry {
    pub limitl: u16,
    pub basel: u16,
    pub basem: u8,
    pub access: u8,
    pub flags_limith: u8,
    pub baseh: u8
}

impl GdtEntry {
    pub const fn new() -> Self {
        GdtEntry {
            limitl: 0,
            basel: 0,
            basem: 0,
            access: 0,
            flags_limith: 0,
            baseh: 0
        }
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.basel = offset as u16;
        self.basem = (offset >> 16) as u8;
        self.baseh = (offset >> 24) as u8;
    }

    pub fn set_access(&mut self, access: GdtAccess) {
        self.access = access.bits;
    }

    pub fn set_flags(&mut self, flags: GdtFlags) {
        self.flags_limith = (self.flags_limith & 0xF) | flags.bits;
    }
}
