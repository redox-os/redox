use core::mem;

use super::sdt::Sdt;

/// The Multiple APIC Descriptor Table
#[derive(Debug)]
pub struct Madt {
    sdt: &'static Sdt,
    pub local_address: u32,
    pub flags: u32
}

impl Madt {
    pub fn new(sdt: &'static Sdt) -> Option<Madt> {
        if &sdt.signature == b"APIC" && sdt.data_len() >= 8 { //Not valid if no local address and flags
            let local_address = unsafe { *(sdt.data_address() as *const u32) };
            let flags = unsafe { *(sdt.data_address() as *const u32).offset(1) };

            Some(Madt {
                sdt: sdt,
                local_address: local_address,
                flags: flags
            })
        } else {
            None
        }
    }

    pub fn iter(&self) -> MadtIter {
        MadtIter {
            sdt: self.sdt,
            i: 8 // Skip local controller address and flags
        }
    }
}

///

/// MADT Local APIC
#[derive(Debug)]
#[repr(packed)]
pub struct MadtLocalApic {
    /// Processor ID
    pub processor: u8,
    /// Local APIC ID
    pub id: u8,
    /// Flags. 1 means that the processor is enabled
    pub flags: u32
}

/// MADT I/O APIC
#[derive(Debug)]
#[repr(packed)]
pub struct MadtIoApic {
    /// I/O APIC ID
    pub id: u8,
    /// reserved
    reserved: u8,
    /// I/O APIC address
    pub address: u32,
    /// Global system interrupt base
    pub gsi_base: u32
}

/// MADT Interrupt Source Override
#[derive(Debug)]
#[repr(packed)]
pub struct MadtIntSrcOverride {
    /// Bus Source
    pub bus_source: u8,
    /// IRQ Source
    pub irq_source: u8,
    /// Global system interrupt base
    pub gsi_base: u32,
    /// Flags
    pub flags: u16
}

/// MADT Entries
#[derive(Debug)]
pub enum MadtEntry {
    LocalApic(&'static MadtLocalApic),
    InvalidLocalApic(usize),
    IoApic(&'static MadtIoApic),
    InvalidIoApic(usize),
    IntSrcOverride(&'static MadtIntSrcOverride),
    InvalidIntSrcOverride(usize),
    Unknown(u8)
}

pub struct MadtIter {
    sdt: &'static Sdt,
    i: usize
}

impl Iterator for MadtIter {
    type Item = MadtEntry;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i + 1 < self.sdt.data_len() {
            let entry_type = unsafe { *(self.sdt.data_address() as *const u8).offset(self.i as isize) };
            let entry_len = unsafe { *(self.sdt.data_address() as *const u8).offset(self.i as isize + 1) } as usize;

            if self.i + entry_len <= self.sdt.data_len() {
                let item = match entry_type {
                    0 => if entry_len == mem::size_of::<MadtLocalApic>() + 2 {
                        MadtEntry::LocalApic(unsafe { &*((self.sdt.data_address() + self.i + 2) as *const MadtLocalApic) })
                    } else {
                        MadtEntry::InvalidLocalApic(entry_len)
                    },
                    1 => if entry_len == mem::size_of::<MadtIoApic>() + 2 {
                        MadtEntry::IoApic(unsafe { &*((self.sdt.data_address() + self.i + 2) as *const MadtIoApic) })
                    } else {
                        MadtEntry::InvalidIoApic(entry_len)
                    },
                    2 => if entry_len == mem::size_of::<MadtIntSrcOverride>() + 2 {
                        MadtEntry::IntSrcOverride(unsafe { &*((self.sdt.data_address() + self.i + 2) as *const MadtIntSrcOverride) })
                    } else {
                        MadtEntry::InvalidIntSrcOverride(entry_len)
                    },
                    _ => MadtEntry::Unknown(entry_type)
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
