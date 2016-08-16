//! # ACPI
//! Code to parse the ACPI tables

use core::mem;

use memory::{Frame, FrameAllocator};
use paging::{entry, ActivePageTable, Page, PhysicalAddress, VirtualAddress};

use self::sdt::SDTHeader;

pub mod sdt;

/// Parse the ACPI tables to gather CPU, interrupt, and timer information
pub unsafe fn init<A>(allocator: &mut A, active_table: &mut ActivePageTable) -> Option<Acpi>
    where A: FrameAllocator
{
    let start_addr = 0xE0000;
    let end_addr = 0xFFFFF;

    // Map all of the ACPI table space
    {
        let start_frame = Frame::containing_address(PhysicalAddress::new(start_addr));
        let end_frame = Frame::containing_address(PhysicalAddress::new(end_addr));
        for frame in Frame::range_inclusive(start_frame, end_frame) {
            active_table.identity_map(frame, entry::PRESENT | entry::NO_EXECUTE, allocator);
        }
    }

    // Search for RSDP
    if let Some(rsdp) = RSDP::search(start_addr, end_addr) {
        println!("{:?}", rsdp);

        let rsdt_frame = Frame::containing_address(PhysicalAddress::new(rsdp.rsdt_address as usize));
        active_table.identity_map(rsdt_frame, entry::PRESENT | entry::NO_EXECUTE, allocator);

        let sdt = unsafe { &*(rsdp.rsdt_address as usize as *const SDTHeader) };
        
        for &c in sdt.signature.iter() {
            print!("{}", c as char);
        }
        println!(" {:?}", sdt);
    }

    None
}

pub struct Acpi;

/// RSDP
#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct RSDP {
    signature: [u8; 8],
    checksum: u8,
    oemid: [u8; 6],
    revision: u8,
    rsdt_address: u32,
    length: u32,
    xsdt_address: u32,
    extended_checksum: u8,
    reserved: [u8; 3]
}

impl RSDP {
    pub fn search(start_addr: usize, end_addr: usize) -> Option<RSDP> {
        for i in 0 .. (end_addr + 1 - start_addr)/mem::size_of::<RSDP>() {
            let mut rsdp = unsafe { &*(start_addr as *const RSDP).offset(i as isize) };
            if &rsdp.signature == b"RSD PTR " {
                return Some(*rsdp);
            }
        }
        None
    }
}
