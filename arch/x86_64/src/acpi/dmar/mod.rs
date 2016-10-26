use core::mem;

use super::sdt::Sdt;
use self::drhd::Drhd;
use memory::Frame;
use paging::{entry, ActivePageTable, PhysicalAddress};

pub mod drhd;

/// The DMA Remapping Table
#[derive(Debug)]
pub struct Dmar {
    sdt: &'static Sdt,
    pub addr_width: u8,
    pub flags: u8,
    _rsv: [u8; 10],
}

impl Dmar {
    pub fn new(sdt: &'static Sdt) -> Option<Dmar> {
        if &sdt.signature == b"DMAR" && sdt.data_len() >= 12 { //Not valid if no local address and flags
            let addr_width = unsafe { *(sdt.data_address() as *const u8) };
            let flags = unsafe { *(sdt.data_address() as *const u8).offset(1) };
            let rsv: [u8; 10] = unsafe { *((sdt.data_address() as *const u8).offset(2) as *const [u8; 10]) };

            Some(Dmar {
                sdt: sdt,
                addr_width: addr_width,
                flags: flags,
                _rsv: rsv,
            })
        } else {
            None
        }
    }

    pub fn iter(&self) -> DmarIter {
        DmarIter {
            sdt: self.sdt,
            i: 12 // Skip address width and flags
        }
    }
}

///

/// DMAR DMA Remapping Hardware Unit Definition
// TODO: Implement iterator on DmarDrhd scope
#[derive(Debug)]
#[repr(packed)]
pub struct DmarDrhd {
    kind: u16,
    length: u16,
    flags: u8,
    _rsv: u8,
    segment: u16,
    base: u64,
}

impl DmarDrhd {
    pub fn get(&self, active_table: &mut ActivePageTable) -> &'static mut Drhd {
        active_table.identity_map(Frame::containing_address(PhysicalAddress::new(self.base as usize)), entry::PRESENT | entry::WRITABLE | entry::NO_EXECUTE);
        unsafe { &mut *(self.base as *mut Drhd) }
    }
}

/// DMAR Reserved Memory Region Reporting
// TODO: Implement iterator on DmarRmrr scope
#[derive(Debug)]
#[repr(packed)]
pub struct DmarRmrr {
    kind: u16,
    length: u16,
    _rsv: u16,
    segment: u16,
    base: u64,
    limit: u64,
}

/// DMAR Root Port ATS Capability Reporting
// TODO: Implement iterator on DmarAtsr scope
#[derive(Debug)]
#[repr(packed)]
pub struct DmarAtsr {
    kind: u16,
    length: u16,
    flags: u8,
    _rsv: u8,
    segment: u16,
}

/// DMAR Remapping Hardware Static Affinity
#[derive(Debug)]
#[repr(packed)]
pub struct DmarRhsa {
    kind: u16,
    length: u16,
    _rsv: u32,
    base: u64,
    domain: u32,
}

/// DMAR ACPI Name-space Device Declaration
// TODO: Implement iterator on DmarAndd object name
#[derive(Debug)]
#[repr(packed)]
pub struct DmarAndd {
    kind: u16,
    length: u16,
    _rsv: [u8; 3],
    acpi_dev: u8,
}

/// DMAR Entries
#[derive(Debug)]
pub enum DmarEntry {
    Drhd(&'static DmarDrhd),
    InvalidDrhd(usize),
    Rmrr(&'static DmarRmrr),
    InvalidRmrr(usize),
    Atsr(&'static DmarAtsr),
    InvalidAtsr(usize),
    Rhsa(&'static DmarRhsa),
    InvalidRhsa(usize),
    Andd(&'static DmarAndd),
    InvalidAndd(usize),
    Unknown(u16)
}

pub struct DmarIter {
    sdt: &'static Sdt,
    i: usize
}

impl Iterator for DmarIter {
    type Item = DmarEntry;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i + 4 <= self.sdt.data_len() {
            let entry_type = unsafe { *((self.sdt.data_address() as *const u8).offset(self.i as isize) as *const u16) };
            let entry_len = unsafe { *((self.sdt.data_address() as *const u8).offset(self.i as isize + 2) as *const u16) } as usize;

            if self.i + entry_len <= self.sdt.data_len() {
                let item = match entry_type {
                    0 => if entry_len >= mem::size_of::<DmarDrhd>() {
                        DmarEntry::Drhd(unsafe { &*((self.sdt.data_address() + self.i) as *const DmarDrhd) })
                    } else {
                        DmarEntry::InvalidDrhd(entry_len)
                    },
                    1 => if entry_len >= mem::size_of::<DmarRmrr>() {
                        DmarEntry::Rmrr(unsafe { &*((self.sdt.data_address() + self.i) as *const DmarRmrr) })
                    } else {
                        DmarEntry::InvalidRmrr(entry_len)
                    },
                    2 => if entry_len >= mem::size_of::<DmarAtsr>() {
                        DmarEntry::Atsr(unsafe { &*((self.sdt.data_address() + self.i) as *const DmarAtsr) })
                    } else {
                        DmarEntry::InvalidAtsr(entry_len)
                    },
                    3 => if entry_len == mem::size_of::<DmarRhsa>() {
                        DmarEntry::Rhsa(unsafe { &*((self.sdt.data_address() + self.i) as *const DmarRhsa) })
                    } else {
                        DmarEntry::InvalidRhsa(entry_len)
                    },
                    4 => if entry_len >= mem::size_of::<DmarAndd>() {
                        DmarEntry::Andd(unsafe { &*((self.sdt.data_address() + self.i) as *const DmarAndd) })
                    } else {
                        DmarEntry::InvalidAndd(entry_len)
                    },
                    _ => DmarEntry::Unknown(entry_type)
                };

                self.i += entry_len;

                Some(item)
            } else {
                None
            }
        } else {
            None
        }
    }
}
