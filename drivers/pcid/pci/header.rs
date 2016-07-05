use std::ops::{Deref, DerefMut};
use std::{slice, mem};

#[derive(Debug, Default)]
#[repr(packed)]
pub struct PciHeader {
    pub vendor_id: u16,
    pub device_id: u16,
    pub command: u16,
    pub status: u16,
    pub revision: u8,
    pub interface: u8,
    pub subclass: u8,
    pub class: u8,
    pub cache_line_size: u8,
    pub latency_timer: u8,
    pub header_type: u8,
    pub bist: u8,
    pub bars: [u32; 6],
    pub cardbus_cis_ptr: u32,
    pub subsystem_vendor_id: u16,
    pub subsystem_id: u16,
    pub expansion_rom_bar: u32,
    pub capabilities: u8,
    pub reserved: [u8; 7],
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub min_grant: u8,
    pub max_latency: u8
}

impl Deref for PciHeader {
    type Target = [u32];
    fn deref(&self) -> &[u32] {
        unsafe { slice::from_raw_parts(self as *const PciHeader as *const u32, mem::size_of::<PciHeader>()/4) as &[u32] }
    }
}

impl DerefMut for PciHeader {
    fn deref_mut(&mut self) -> &mut [u32] {
        unsafe { slice::from_raw_parts_mut(self as *mut PciHeader as *mut u32, mem::size_of::<PciHeader>()/4) as &mut [u32] }
    }
}
