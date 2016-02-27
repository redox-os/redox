#[derive(Copy, Clone, Debug, Default)]
#[repr(packed)]
pub struct Regs {
    pub ax: usize,
    pub bx: usize,
    pub cx: usize,
    pub dx: usize,
    pub di: usize,
    pub si: usize,
    pub r8: usize,
    pub r9: usize,
    pub r10: usize,
    pub r11: usize,
    pub r12: usize,
    pub r13: usize,
    pub r14: usize,
    pub r15: usize,
    pub bp: usize,
    pub ip: usize,
    pub cs: usize,
    pub flags: usize,
    pub sp: usize,
    pub ss: usize,
}
