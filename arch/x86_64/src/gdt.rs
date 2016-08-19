//! Global descriptor table

use core::mem;
use x86::dtables::{self, DescriptorTablePointer};
use x86::segmentation::{self, SegmentSelector};
use x86::task::{self, TaskStateSegment};

pub const GDT_NULL: usize = 0;
pub const GDT_KERNEL_CODE: usize = 1;
pub const GDT_KERNEL_DATA: usize = 2;
pub const GDT_USER_CODE: usize = 3;
pub const GDT_USER_DATA: usize = 4;
pub const GDT_USER_TLS: usize = 5;
pub const GDT_TSS: usize = 6;
pub const GDT_TSS_HIGH: usize = 7;

pub const GDT_A_PRESENT: u8 = 1 << 7;
pub const GDT_A_RING_0: u8 = 0 << 5;
pub const GDT_A_RING_1: u8 = 1 << 5;
pub const GDT_A_RING_2: u8 = 2 << 5;
pub const GDT_A_RING_3: u8 = 3 << 5;
pub const GDT_A_SYSTEM: u8 = 1 << 4;
pub const GDT_A_EXECUTABLE: u8 = 1 << 3;
pub const GDT_A_CONFORMING: u8 = 1 << 2;
pub const GDT_A_PRIVILEGE: u8 = 1 << 1;
pub const GDT_A_DIRTY: u8 = 1;

pub const GDT_A_TSS_AVAIL: u8 = 0x9;
pub const GDT_A_TSS_BUSY: u8 = 0xB;

pub const GDT_F_PAGE_SIZE: u8 = 1 << 7;
pub const GDT_F_PROTECTED_MODE: u8 = 1 << 6;
pub const GDT_F_LONG_MODE: u8 = 1 << 5;

pub static mut GDTR: DescriptorTablePointer = DescriptorTablePointer {
    limit: 0,
    base: 0
};

pub static mut GDT: [GdtEntry; 8] = [
    // Null
    GdtEntry::new(0, 0, 0, 0),
    // Kernel code
    GdtEntry::new(0, 0, GDT_A_PRESENT | GDT_A_RING_0 | GDT_A_SYSTEM | GDT_A_EXECUTABLE | GDT_A_PRIVILEGE, GDT_F_LONG_MODE),
    // Kernel data
    GdtEntry::new(0, 0, GDT_A_PRESENT | GDT_A_RING_0 | GDT_A_SYSTEM | GDT_A_PRIVILEGE, GDT_F_LONG_MODE),
    // User code
    GdtEntry::new(0, 0, GDT_A_PRESENT | GDT_A_RING_3 | GDT_A_SYSTEM | GDT_A_EXECUTABLE | GDT_A_PRIVILEGE, GDT_F_LONG_MODE),
    // User data
    GdtEntry::new(0, 0, GDT_A_PRESENT | GDT_A_RING_3 | GDT_A_SYSTEM | GDT_A_PRIVILEGE, GDT_F_LONG_MODE),
    //TODO: User TLS
    GdtEntry::new(0, 0, GDT_A_PRESENT | GDT_A_RING_3 | GDT_A_SYSTEM | GDT_A_PRIVILEGE, GDT_F_LONG_MODE),
    //TODO: TSS
    GdtEntry::new(0, 0, 0 , 0),
    // TSS must be 16 bytes long, twice the normal size
    GdtEntry::new(0, 0, 0, 0),
];

pub static mut TSS: TaskStateSegment = TaskStateSegment {
    reserved: 0,
    rsp: [0; 3],
    reserved2: 0,
    ist: [0; 7],
    reserved3: 0,
    reserved4: 0,
    iomap_base: 0xFFFF
};

pub unsafe fn init() {
    GDTR.limit = (GDT.len() * mem::size_of::<GdtEntry>() - 1) as u16;
    GDTR.base = GDT.as_ptr() as u64;

    GDT[GDT_TSS] = GdtEntry::new(&TSS as *const _ as u32, mem::size_of::<TaskStateSegment>() as u32, GDT_A_PRESENT | GDT_A_RING_3 | GDT_A_TSS_AVAIL, 0);

    init_ap();
}

pub unsafe fn init_ap() {
    dtables::lgdt(&GDTR);

    segmentation::load_cs(SegmentSelector::new(GDT_KERNEL_CODE as u16));
    segmentation::load_ds(SegmentSelector::new(GDT_KERNEL_DATA as u16));
    segmentation::load_es(SegmentSelector::new(GDT_KERNEL_DATA as u16));
    segmentation::load_fs(SegmentSelector::new(GDT_KERNEL_DATA as u16));
    segmentation::load_gs(SegmentSelector::new(GDT_KERNEL_DATA as u16));
    segmentation::load_ss(SegmentSelector::new(GDT_KERNEL_DATA as u16));

    //TODO: Seperate TSS for each processor task::load_ltr(SegmentSelector::new(GDT_TSS as u16));
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct GdtEntry {
    pub limitl: u16,
    pub offsetl: u16,
    pub offsetm: u8,
    pub access: u8,
    pub flags_limith: u8,
    pub offseth: u8
}

impl GdtEntry {
    pub const fn new(offset: u32, limit: u32, access: u8, flags: u8) -> Self {
        GdtEntry {
            limitl: limit as u16,
            offsetl: offset as u16,
            offsetm: (offset >> 16) as u8,
            access: access,
            flags_limith: flags & 0xF0 | ((limit >> 16) as u8) & 0x0F,
            offseth: (offset >> 24) as u8
        }
    }
}
