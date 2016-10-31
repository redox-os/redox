use device::local_apic::LOCAL_APIC;

interrupt!(ipi, {
    LOCAL_APIC.eoi();
});
