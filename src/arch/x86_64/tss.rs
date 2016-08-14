/// The Task State Segment.
#[repr(C, packed)]
#[derive(Debug, Default, Clone)]
pub struct Tss {
    /// Reserved.
    pub _reserved1: u32,
    /// The stack-pointers (reg RSP) for the IO privilege level 0 through 2.
    pub rsp: [u64; 3],
    /// Reserved.
    pub _reserved2: u32,
    /// Reserved.
    pub _reserved3: u32,
    pub ist: [u64; 7],
    /// Reserved.
    pub _reserved4: u32,
    /// Reserved.
    pub _reserved5: u32,
    // Reserved.
    pub reserved6: u16,
    /// The offset to the IOPB.
    pub iomap_base: u16,
}
