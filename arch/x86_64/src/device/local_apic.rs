use core::intrinsics::{volatile_load, volatile_store};
use x86::cpuid::CpuId;
use x86::msr::*;

use memory::Frame;
use paging::{entry, ActivePageTable, PhysicalAddress, Page, VirtualAddress};

pub static mut LOCAL_APIC: LocalApic = LocalApic {
    address: 0,
    x2: false
};

pub unsafe fn init(active_table: &mut ActivePageTable) {
    LOCAL_APIC.init(active_table);
}

pub unsafe fn init_ap() {
    LOCAL_APIC.init_ap();
}

/// Local APIC
pub struct LocalApic {
    pub address: usize,
    pub x2: bool
}

impl LocalApic {
    unsafe fn init(&mut self, active_table: &mut ActivePageTable) {
        self.address = (rdmsr(IA32_APIC_BASE) as usize & 0xFFFF0000) + ::KERNEL_OFFSET;
        self.x2 = CpuId::new().get_feature_info().unwrap().has_x2apic();

        if ! self.x2 {
            let page = Page::containing_address(VirtualAddress::new(self.address));
            let frame = Frame::containing_address(PhysicalAddress::new(self.address - ::KERNEL_OFFSET));
            active_table.map_to(page, frame, entry::PRESENT | entry::WRITABLE | entry::NO_EXECUTE);
        }

        self.init_ap();
    }

    unsafe fn init_ap(&mut self) {
        if self.x2 {
            wrmsr(IA32_APIC_BASE, rdmsr(IA32_APIC_BASE) | 1 << 10);
            wrmsr(IA32_X2APIC_SIVR, 0x100);
        } else {
            self.write(0xF0, 0x100);
        }
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        volatile_load((self.address + reg as usize) as *const u32)
    }

    unsafe fn write(&mut self, reg: u32, value: u32) {
        volatile_store((self.address + reg as usize) as *mut u32, value);
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

    pub fn ipi(&mut self, apic_id: usize) {
        let mut icr = 0x4040;
        if self.x2 {
            icr |= (apic_id as u64) << 32;
        } else {
            icr |= (apic_id as u64) << 56;
        }
        self.set_icr(icr);
    }

    pub unsafe fn eoi(&mut self) {
        if self.x2 {
            wrmsr(IA32_X2APIC_EOI, 0);
        } else {
            self.write(0xB0, 0);
        }
    }
}
