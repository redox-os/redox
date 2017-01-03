#[repr(packed)]
pub struct DrhdFault {
    pub sts: u32,
    pub ctrl: u32,
    pub data: u32,
    pub addr: [u32; 2],
    _rsv: [u64; 2],
    pub log: u64,
}

#[repr(packed)]
pub struct DrhdProtectedMemory {
    pub en: u32,
    pub low_base: u32,
    pub low_limit: u32,
    pub high_base: u64,
    pub high_limit: u64,
}

#[repr(packed)]
pub struct DrhdInvalidation {
    pub queue_head: u64,
    pub queue_tail: u64,
    pub queue_addr: u64,
    _rsv: u32,
    pub cmpl_sts: u32,
    pub cmpl_ctrl: u32,
    pub cmpl_data: u32,
    pub cmpl_addr: [u32; 2],
}

#[repr(packed)]
pub struct DrhdPageRequest {
    pub queue_head: u64,
    pub queue_tail: u64,
    pub queue_addr: u64,
    _rsv: u32,
    pub sts: u32,
    pub ctrl: u32,
    pub data: u32,
    pub addr: [u32; 2],
}

#[repr(packed)]
pub struct DrhdMtrrVariable {
    pub base: u64,
    pub mask: u64,
}

#[repr(packed)]
pub struct DrhdMtrr {
    pub cap: u64,
    pub def_type: u64,
    pub fixed: [u64; 11],
    pub variable: [DrhdMtrrVariable; 10],
}

#[repr(packed)]
pub struct Drhd {
    pub version: u32,
    _rsv: u32,
    pub cap: u64,
    pub ext_cap: u64,
    pub gl_cmd: u32,
    pub gl_sts: u32,
    pub root_table: u64,
    pub ctx_cmd: u64,
    _rsv1: u32,
    pub fault: DrhdFault,
    _rsv2: u32,
    pub pm: DrhdProtectedMemory,
    pub invl: DrhdInvalidation,
    _rsv3: u64,
    pub intr_table: u64,
    pub page_req: DrhdPageRequest,
    pub mtrr: DrhdMtrr,
}
