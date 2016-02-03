#[derive(Copy, Clone, Default)]
#[repr(packed)]
pub struct Regs {
    pub ax: usize,
    pub bx: usize,
    pub cx: usize,
    pub dx: usize,
    pub di: usize,
    pub si: usize,
    pub bp: usize,
    pub ip: usize,
    pub cs: usize,
    pub flags: usize,
    pub sp: usize,
    pub ss: usize,
}
