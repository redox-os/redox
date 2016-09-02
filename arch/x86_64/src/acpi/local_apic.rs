use core::intrinsics::{volatile_load, volatile_store};
use x86::cpuid::CpuId;
use x86::msr::*;

use memory::Frame;
use paging::{entry, ActivePageTable, PhysicalAddress};

/// Local APIC
pub struct LocalApic {
    pub address: u32,
    pub x2: bool
}

impl LocalApic {
    pub fn new(active_table: &mut ActivePageTable) -> Self {
        let mut apic = LocalApic {
            address: (unsafe { rdmsr(IA32_APIC_BASE) as u32 } & 0xFFFF0000),
            x2: false
        };

        unsafe { wrmsr(IA32_APIC_BASE, rdmsr(IA32_APIC_BASE) & !(1 << 11 | 1 << 10)) };

        unsafe { wrmsr(IA32_APIC_BASE, rdmsr(IA32_APIC_BASE) | 1 << 11) };

        if CpuId::new().get_feature_info().unwrap().has_x2apic() {
            unsafe { wrmsr(IA32_APIC_BASE, rdmsr(IA32_APIC_BASE) | 1 << 10) };
            apic.x2 = true;
        } else {
            active_table.identity_map(Frame::containing_address(PhysicalAddress::new(apic.address as usize)), entry::PRESENT | entry::WRITABLE | entry::NO_EXECUTE);
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
