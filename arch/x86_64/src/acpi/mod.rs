//! # ACPI
//! Code to parse the ACPI tables

use core::intrinsics::{atomic_load, atomic_store};
use core::sync::atomic::Ordering;

use device::local_apic::LOCAL_APIC;
use interrupt;
use memory::{allocate_frames, Frame};
use paging::{entry, ActivePageTable, Page, PhysicalAddress, VirtualAddress};
use start::{kstart_ap, CPU_COUNT, AP_READY};

use self::dmar::{Dmar, DmarEntry};
use self::madt::{Madt, MadtEntry};
use self::rsdt::Rsdt;
use self::sdt::Sdt;
use self::xsdt::Xsdt;

pub mod dmar;
pub mod madt;
pub mod rsdt;
pub mod sdt;
pub mod xsdt;

const TRAMPOLINE: usize = 0x7E00;
const AP_STARTUP: usize = TRAMPOLINE + 512;

pub fn init_sdt(sdt: &'static Sdt, active_table: &mut ActivePageTable) {
    print!("  ");
    for &c in sdt.signature.iter() {
        print!("{}", c as char);
    }

    if let Some(madt) = Madt::new(sdt) {
        println!(": {:>08X}: {}", madt.local_address, madt.flags);

        let mut local_apic = unsafe { &mut LOCAL_APIC };

        let me = local_apic.id() as u8;

        if local_apic.x2 {
            println!("    X2APIC {}", me);
        } else {
            println!("    XAPIC {}: {:>08X}", me, local_apic.address);
        }

        let trampoline_frame = Frame::containing_address(PhysicalAddress::new(TRAMPOLINE));
        let trampoline_page = Page::containing_address(VirtualAddress::new(TRAMPOLINE));

        // Map trampoline
        active_table.map_to(trampoline_page, trampoline_frame, entry::PRESENT | entry::WRITABLE);
        active_table.flush(trampoline_page);

        for madt_entry in madt.iter() {
            println!("      {:?}", madt_entry);
            match madt_entry {
                MadtEntry::LocalApic(ap_local_apic) => if ap_local_apic.id == me {
                    println!("        This is my local APIC");
                } else {
                    if ap_local_apic.flags & 1 == 1 {
                        // Increase CPU ID
                        CPU_COUNT.fetch_add(1, Ordering::SeqCst);

                        // Allocate a stack
                        let stack_start = allocate_frames(64).expect("no more frames in acpi stack_start").start_address().get() + ::KERNEL_OFFSET;
                        let stack_end = stack_start + 64 * 4096;

                        let ap_ready = TRAMPOLINE as *mut u64;
                        let ap_cpu_id = unsafe { ap_ready.offset(1) };
                        let ap_page_table = unsafe { ap_ready.offset(2) };
                        let ap_stack_start = unsafe { ap_ready.offset(3) };
                        let ap_stack_end = unsafe { ap_ready.offset(4) };
                        let ap_code = unsafe { ap_ready.offset(5) };

                        // Set the ap_ready to 0, volatile
                        unsafe { atomic_store(ap_ready, 0) };
                        unsafe { atomic_store(ap_cpu_id, ap_local_apic.id as u64) };
                        unsafe { atomic_store(ap_page_table, active_table.address() as u64) };
                        unsafe { atomic_store(ap_stack_start, stack_start as u64) };
                        unsafe { atomic_store(ap_stack_end, stack_end as u64) };
                        unsafe { atomic_store(ap_code, kstart_ap as u64) };
                        AP_READY.store(false, Ordering::SeqCst);

                        print!("        AP {}:", ap_local_apic.id);

                        // Send INIT IPI
                        {
                            let mut icr = 0x4500;
                            if local_apic.x2 {
                                icr |= (ap_local_apic.id as u64) << 32;
                            } else {
                                icr |= (ap_local_apic.id as u64) << 56;
                            }
                            print!(" IPI...");
                            local_apic.set_icr(icr);
                        }

                        // Send START IPI
                        {
                            //Start at 0x0800:0000 => 0x8000. Hopefully the bootloader code is still there
                            let ap_segment = (AP_STARTUP >> 12) & 0xFF;
                            let mut icr = 0x4600 | ap_segment as u64;

                            if local_apic.x2 {
                                icr |= (ap_local_apic.id as u64) << 32;
                            } else {
                                icr |= (ap_local_apic.id as u64) << 56;
                            }

                            print!(" SIPI...");
                            local_apic.set_icr(icr);
                        }

                        // Wait for trampoline ready
                        print!(" Wait...");
                        while unsafe { atomic_load(ap_ready) } == 0 {
                            interrupt::pause();
                        }
                        print!(" Trampoline...");
                        while ! AP_READY.load(Ordering::SeqCst) {
                            interrupt::pause();
                        }
                        println!(" Ready");

                        active_table.flush_all();
                    } else {
                        println!("        CPU Disabled");
                    }
                },
                _ => ()
            }
        }

        // Unmap trampoline
        active_table.unmap(trampoline_page);
        active_table.flush(trampoline_page);
    } else if let Some(dmar) = Dmar::new(sdt) {
        println!(": {}: {}", dmar.addr_width, dmar.flags);

        for dmar_entry in dmar.iter() {
            println!("      {:?}", dmar_entry);
            match dmar_entry {
                DmarEntry::Drhd(dmar_drhd) => {
                    let drhd = dmar_drhd.get(active_table);

                    println!("VER: {:X}", drhd.version);
                    println!("CAP: {:X}", drhd.cap);
                    println!("EXT_CAP: {:X}", drhd.ext_cap);
                    println!("GCMD: {:X}", drhd.gl_cmd);
                    println!("GSTS: {:X}", drhd.gl_sts);
                    println!("RT: {:X}", drhd.root_table);
                },
                _ => ()
            }
        }
    } else {
        println!(": Unknown");
    }
}

/// Parse the ACPI tables to gather CPU, interrupt, and timer information
pub unsafe fn init(active_table: &mut ActivePageTable) -> Option<Acpi> {
    let start_addr = 0xE0000;
    let end_addr = 0xFFFFF;

    // Map all of the ACPI RSDP space
    {
        let start_frame = Frame::containing_address(PhysicalAddress::new(start_addr));
        let end_frame = Frame::containing_address(PhysicalAddress::new(end_addr));
        for frame in Frame::range_inclusive(start_frame, end_frame) {
            let page = Page::containing_address(VirtualAddress::new(frame.start_address().get()));
            active_table.map_to(page, frame, entry::PRESENT | entry::NO_EXECUTE);
            active_table.flush(page);
        }
    }

    // Search for RSDP
    if let Some(rsdp) = RSDP::search(start_addr, end_addr) {
        let get_sdt = |sdt_address: usize, active_table: &mut ActivePageTable| -> (&'static Sdt, bool) {
            let mapped = if active_table.translate_page(Page::containing_address(VirtualAddress::new(sdt_address))).is_none() {
                let sdt_frame = Frame::containing_address(PhysicalAddress::new(sdt_address));
                let sdt_page = Page::containing_address(VirtualAddress::new(sdt_address));
                active_table.map_to(sdt_page, sdt_frame, entry::PRESENT | entry::NO_EXECUTE);
                active_table.flush(sdt_page);
                true
            } else {
                false
            };
            (&*(sdt_address as *const Sdt), mapped)
        };

        let drop_sdt = |sdt: &'static Sdt, mapped: bool, active_table: &mut ActivePageTable| {
            let sdt_address = sdt as *const Sdt as usize;
            drop(sdt);
            if mapped {
                let sdt_page = Page::containing_address(VirtualAddress::new(sdt_address));
                active_table.unmap(sdt_page);
                active_table.flush(sdt_page);
            }
        };

        let (rxsdt, rxmapped) = get_sdt(rsdp.sdt_address(), active_table);

        for &c in rxsdt.signature.iter() {
            print!("{}", c as char);
        }
        println!(":");
        if let Some(rsdt) = Rsdt::new(rxsdt) {
            for sdt_address in rsdt.iter() {
                let (sdt, mapped) = get_sdt(sdt_address, active_table);
                init_sdt(sdt, active_table);
                drop_sdt(sdt, mapped, active_table);
            }
        } else if let Some(xsdt) = Xsdt::new(rxsdt) {
            for sdt_address in xsdt.iter() {
                let (sdt, mapped) = get_sdt(sdt_address, active_table);
                init_sdt(sdt, active_table);
                drop_sdt(sdt, mapped, active_table);
            }
        } else {
            println!("UNKNOWN RSDT OR XSDT SIGNATURE");
        }

        drop_sdt(rxsdt, rxmapped, active_table);
    } else {
        println!("NO RSDP FOUND");
    }

    // Unmap all of the ACPI RSDP space
    {
        let start_frame = Frame::containing_address(PhysicalAddress::new(start_addr));
        let end_frame = Frame::containing_address(PhysicalAddress::new(end_addr));
        for frame in Frame::range_inclusive(start_frame, end_frame) {
            let page = Page::containing_address(VirtualAddress::new(frame.start_address().get()));
            active_table.unmap(page);
            active_table.flush(page);
        }
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
