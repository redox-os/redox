//! # ACPI
//! Code to parse the ACPI tables

use memory::{Frame, FrameAllocator};
use paging::{entry, ActivePageTable, Page, PhysicalAddress, VirtualAddress};

use self::rsdt::RSDT;
use self::sdt::SDT;
use self::xsdt::XSDT;

pub mod rsdt;
pub mod sdt;
pub mod xsdt;

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
            if active_table.translate_page(Page::containing_address(VirtualAddress::new(frame.start_address().get()))).is_none() {
                active_table.identity_map(frame, entry::PRESENT | entry::NO_EXECUTE, allocator);
            }
        }
    }

    // Search for RSDP
    if let Some(rsdp) = RSDP::search(start_addr, end_addr) {
        println!("{:?}", rsdp);

        let mut get_sdt = |sdt_address: usize| -> &'static SDT {
            if active_table.translate_page(Page::containing_address(VirtualAddress::new(sdt_address))).is_none() {
                let sdt_frame = Frame::containing_address(PhysicalAddress::new(sdt_address));
                active_table.identity_map(sdt_frame, entry::PRESENT | entry::NO_EXECUTE, allocator);
            }
            unsafe { &*(sdt_address as *const SDT) }
        };

        let rxsdt = get_sdt(rsdp.sdt_address());

        for &c in rxsdt.signature.iter() {
            print!("{}", c as char);
        }
        println!(":");
        if let Some(rsdt) = RSDT::new(rxsdt) {
            println!("{:?}", rsdt);
            for sdt_address in rsdt.iter() {
                let sdt = get_sdt(sdt_address);
                for &c in sdt.signature.iter() {
                    print!("{}", c as char);
                }
                println!(":");
                println!("{:?}", sdt);
            }
        } else if let Some(xsdt) = XSDT::new(rxsdt) {
            println!("{:?}", xsdt);
            for sdt_address in xsdt.iter() {
                let sdt = get_sdt(sdt_address);
                for &c in sdt.signature.iter() {
                    print!("{}", c as char);
                }
                println!(":");
                println!("{:?}", sdt);
            }
        } else {
            println!("UNKNOWN RSDT OR XSDT SIGNATURE");
        }
    } else {
        println!("NO RSDP FOUND");
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
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3]
}

impl RSDP {
    /// Search for the RSDP
    pub fn search(start_addr: usize, end_addr: usize) -> Option<RSDP> {
        for i in 0 .. (end_addr + 1 - start_addr)/16 {
            let mut rsdp = unsafe { &*((start_addr + i * 16) as *const RSDP) };
            if &rsdp.signature == b"RSD PTR " {
                return Some(*rsdp);
            }
        }
        None
    }

    /// Get the RSDT or XSDT address
    pub fn sdt_address(&self) -> usize {
        if self.revision >= 2 {
            self.xsdt_address as usize
        } else {
            self.rsdt_address as usize
        }
    }
}
