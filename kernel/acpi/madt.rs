use super::SDTHeader;

use collections::vec::Vec;

use core::mem::size_of;
use core::ptr;

const ENTRY_LOCAL_APIC: u8 = 0;
#[repr(packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LocalApic {
    pub entry_type: u8,
    pub entry_len: u8,
    pub processor: u8,
    pub id: u8,
    pub flags: u32,
}

const ENTRY_IO_APIC: u8 = 1;
#[repr(packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct IoApic {
    pub entry_type: u8,
    pub entry_len: u8,
    pub id: u8,
    pub reserved: u8,
    pub address: u32,
    pub gsi_base: u32,
}

const ENTRY_INT_SOURCE_OVERRIDE: u8 = 2;
#[repr(packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct IntSourceOverride {
    pub entry_type: u8,
    pub entry_len: u8,
    pub bus_source: u8,
    pub irq_source: u8,
    pub gsi: u32,
    pub flags: u16,
}

#[repr(packed)]
#[derive(Clone, Debug)]
pub struct MADT {
    pub header: &'static SDTHeader,
    pub local_apic_address: u32,
    pub flags: u32,
    pub local_apics: Vec<LocalApic>,
    pub io_apics: Vec<IoApic>,
    pub int_source_overrides: Vec<IntSourceOverride>,
}

impl MADT {
    pub fn new(header: &'static SDTHeader) -> Option<Self> {
        if header.valid("APIC") {
            let data: &'static [u8] = header.data();

            let mut madt = MADT {
                header: header,
                local_apic_address: if data.len() >= 4 {
                    unsafe { ptr::read(data.as_ptr().offset(0) as *const u32) }
                } else {
                    0
                },
                flags: if data.len() >= 8 {
                    unsafe { ptr::read(data.as_ptr().offset(4) as *const u32) }
                } else {
                    0
                },
                local_apics: Vec::new(),
                io_apics: Vec::new(),
                int_source_overrides: Vec::new(),
            };

            let mut i = 8;
            while i + 1 < data.len() {
                let entry_type = data[i];
                let entry_length = data[i + 1] as usize;
                if i + entry_length <= data.len() {
                    match entry_type {
                        ENTRY_LOCAL_APIC => {
                            if entry_length == size_of::<LocalApic>() {
                                madt.local_apics.push(unsafe {
                                    ptr::read(data.as_ptr().offset(i as isize) as *const LocalApic)
                                });
                            } else {
                                syslog_debug!("MADT: Unknown LocalApic length: {}", entry_length);
                            }
                        }
                        ENTRY_IO_APIC => {
                            if entry_length == size_of::<IoApic>() {
                                madt.io_apics.push(unsafe {
                                    ptr::read(data.as_ptr().offset(i as isize) as *const IoApic)
                                });
                            } else {
                                syslog_debug!("MADT: Unknown IoApic length: {}", entry_length);
                            }
                        }
                        ENTRY_INT_SOURCE_OVERRIDE => {
                            if entry_length == size_of::<IntSourceOverride>() {
                                madt.int_source_overrides.push(unsafe {
                                ptr::read(data.as_ptr().offset(i as isize) as *const IntSourceOverride)
                            });
                            } else {
                                syslog_debug!("MADT: Unknown IntSourceOverride length: {}",
                                         entry_length);
                            }
                        }
                        _ => {
                            syslog_debug!("MADT: Unknown entry type: {}, length {}",
                                     entry_type,
                                     entry_length)
                        }
                    }
                }
                i += entry_length;
            }

            Some(madt)
        } else {
            None
        }
    }
}
