//! # ACPI
//! Code to parse the ACPI tables

use memory::{Frame, FrameAllocator};
use paging::{entry, ActivePageTable, Page, PhysicalAddress, VirtualAddress};

use self::local_apic::{LocalApic, LocalApicIcr};
use self::madt::{Madt, MadtEntry};
use self::rsdt::Rsdt;
use self::sdt::Sdt;
use self::xsdt::Xsdt;

pub mod local_apic;
pub mod madt;
pub mod rsdt;
pub mod sdt;
pub mod xsdt;

pub fn init_sdt(sdt: &'static Sdt) {
    print!("  ");
    for &c in sdt.signature.iter() {
        print!("{}", c as char);
    }
    println!(":");

    if let Some(madt) = Madt::new(sdt) {
        println!("    {:>016X}: {}", madt.local_address, madt.flags);

        let mut local_apic = LocalApic::new();

        let me = local_apic.id() as u8;

        for madt_entry in madt.iter() {
            println!("      {:?}", madt_entry);
            match madt_entry {
                MadtEntry::LocalApic(asp_local_apic) => if asp_local_apic.id == me {
                    println!("        This is my local APIC");
                } else {
                    if asp_local_apic.flags & 1 == 1 {
                        {
                            let icr = 0x00004500 | (asp_local_apic.id as u64) << 32;
                            println!("        Sending IPI to {}: {:>016X} {:?}", asp_local_apic.id, icr, LocalApicIcr::from_bits(icr));
                            local_apic.set_icr(icr);
                        }
                        {
                            let icr = 0x00004600 | (asp_local_apic.id as u64) << 32;
                            println!("        Sending SIPI to {}: {:>016X} {:?}", asp_local_apic.id, icr, LocalApicIcr::from_bits(icr));
                            local_apic.set_icr(icr);
                        }
                    } else {
                        println!("        CPU Disabled");
                    }
                },
                _ => ()
            }
        }
    }else {
        println!("    {:?}", sdt);
    }
}

/// Parse the ACPI tables to gather CPU, interrupt, and timer information
pub unsafe fn init<A>(allocator: &mut A, active_table: &mut ActivePageTable) -> Option<Acpi>
    where A: FrameAllocator
{
    // Stupidity of enormous proportion. Write the halt opcode to the 0'th physical address
    // so that START IPI's can halt the processor
    {
        if active_table.translate_page(Page::containing_address(VirtualAddress::new(0))).is_none() {
            active_table.identity_map(Frame::containing_address(PhysicalAddress::new(0)), entry::PRESENT | entry::WRITABLE, allocator);
        }
        unsafe { *(0 as *mut u8) = 0xF4 };
    }

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

        let mut get_sdt = |sdt_address: usize| -> &'static Sdt {
            if active_table.translate_page(Page::containing_address(VirtualAddress::new(sdt_address))).is_none() {
                let sdt_frame = Frame::containing_address(PhysicalAddress::new(sdt_address));
                active_table.identity_map(sdt_frame, entry::PRESENT | entry::NO_EXECUTE, allocator);
            }
            &*(sdt_address as *const Sdt)
        };

        let rxsdt = get_sdt(rsdp.sdt_address());

        for &c in rxsdt.signature.iter() {
            print!("{}", c as char);
        }
        println!(":");
        if let Some(rsdt) = Rsdt::new(rxsdt) {
            for sdt_address in rsdt.iter() {
                init_sdt(get_sdt(sdt_address));
            }
        } else if let Some(xsdt) = Xsdt::new(rxsdt) {
            for sdt_address in xsdt.iter() {
                init_sdt(get_sdt(sdt_address));
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
            let rsdp = unsafe { &*((start_addr + i * 16) as *const RSDP) };
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
