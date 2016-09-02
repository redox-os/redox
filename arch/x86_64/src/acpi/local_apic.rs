use core::intrinsics::{volatile_load, volatile_store};
use x86::cpuid::CpuId;
use x86::msr::*;

use memory::Frame;
use paging::{entry, ActivePageTable, PhysicalAddress};

bitflags! {
    pub flags LocalApicIcr: u64 {
        const ICR_VECTOR = 0xFF,

        const ICR_FIXED = 0b000 << 8,
        const ICR_SMI = 0b010 << 8,
        const ICR_NMI = 0b100 << 8,
        const ICR_INIT = 0b101 << 8,
        const ICR_START = 0b110 << 8,

        const ICR_PHYSICAL = 0 << 11,
        const ICR_LOGICAL = 1 << 11,

        const ICR_DEASSERT = 0 << 14,
        const ICR_ASSERT = 1 << 14,

        const ICR_EDGE = 0 << 15,
        const ICR_LEVEL = 1 << 15,

        const ICR_DESTINATION = 0b1111 << 56,
    }
}

/// Local APIC
#[repr(packed)]
pub struct LocalApic {
    address: u32,
    pub x2: bool
}

impl LocalApic {
    pub fn new(active_table: &mut ActivePageTable) -> Self {
        let mut apic = LocalApic {
            address: (unsafe { rdmsr(IA32_APIC_BASE) as u32 } & 0xFFFF0000),
            x2: false
        };

        println!("APIC BASE: {:>08X}", apic.address);

        unsafe { wrmsr(IA32_APIC_BASE, rdmsr(IA32_APIC_BASE) & !(1 << 11 | 1 << 10)) };

        unsafe { wrmsr(IA32_APIC_BASE, rdmsr(IA32_APIC_BASE) | 1 << 11) };

        if CpuId::new().get_feature_info().unwrap().has_x2apic() {
            unsafe { wrmsr(IA32_APIC_BASE, rdmsr(IA32_APIC_BASE) | 1 << 10) };
            apic.x2 = true;
            println!("X2APIC {:X}", unsafe { rdmsr(IA32_APIC_BASE) });
        } else {
            active_table.identity_map(Frame::containing_address(PhysicalAddress::new(apic.address as usize)), entry::PRESENT | entry::WRITABLE | entry::NO_EXECUTE);

            println!("XAPIC {:X}", unsafe { rdmsr(IA32_APIC_BASE) });
        }

        apic
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        volatile_load((self.address + reg) as *const u32)
    }

    unsafe fn write(&self, reg: u32, value: u32) {
        volatile_store((self.address + reg) as *mut u32, value);
    }

    pub fn id(&self) -> u32 {
        if self.x2 {
            unsafe { rdmsr(IA32_X2APIC_APICID) as u32 }
        } else {
            unsafe { self.read(0x20) }
        }
    }

    pub fn version(&self) -> u32 {
        if self.x2 {
            unsafe { rdmsr(IA32_X2APIC_VERSION) as u32 }
        } else {
            unsafe { self.read(0x30) }
        }
    }

    pub fn icr(&self) -> u64 {
        if self.x2 {
            unsafe { rdmsr(IA32_X2APIC_ICR) }
        } else {
            unsafe {
                (self.read(0x310) as u64) << 32 | self.read(0x300) as u64
            }
        }
    }

    pub fn set_icr(&mut self, value: u64) {
        if self.x2 {
            unsafe { wrmsr(IA32_X2APIC_ICR, value); }
        } else {
            unsafe {
                while self.read(0x300) & 1 << 12 == 1 << 12 {}
                self.write(0x310, (value >> 32) as u32);
                self.write(0x300, value as u32);
                while self.read(0x300) & 1 << 12 == 1 << 12 {}
            }
        }
    }
}
