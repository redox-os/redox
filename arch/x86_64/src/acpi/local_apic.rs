use x86::msr::*;

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
pub struct LocalApic;

impl LocalApic {
    pub fn new() -> Self {
        unsafe { wrmsr(IA32_APIC_BASE, rdmsr(IA32_APIC_BASE) | 1 << 10) }
        LocalApic
    }

    pub fn id(&self) -> u32 {
        unsafe { rdmsr(IA32_X2APIC_APICID) as u32 }
    }

    pub fn version(&self) -> u32 {
        unsafe { rdmsr(IA32_X2APIC_VERSION) as u32 }
    }

    pub fn icr(&self) -> u64 {
        unsafe { rdmsr(IA32_X2APIC_ICR) }
    }

    pub fn set_icr(&mut self, value: u64) {
        unsafe { wrmsr(IA32_X2APIC_ICR, value) }
    }
}
