extern crate raw_cpuid;

use core::fmt::{Result, Write};

use self::raw_cpuid::CpuId;

pub fn cpu_info<W: Write>(w: &mut W) -> Result {
    let cpuid = CpuId::new();

    if let Some(info) = cpuid.get_vendor_info() {
        write!(w, "Vendor: {}\n", info.as_string())?;
    }

    if let Some(info) = cpuid.get_extended_function_info() {
        if let Some(brand) = info.processor_brand_string() {
            write!(w, "Model: {}\n", brand)?;
        }
    }

    if let Some(info) = cpuid.get_processor_frequency_info() {
        write!(w, "CPU Base MHz: {}\n", info.processor_base_frequency())?;
        write!(w, "CPU Max MHz: {}\n", info.processor_max_frequency())?;
        write!(w, "Bus MHz: {}\n", info.bus_frequency())?;
    }

    write!(w, "Features:")?;

    if let Some(info) = cpuid.get_feature_info() {
        if info.has_fpu() { write!(w, " fpu")? };
        if info.has_vme() { write!(w, " vme")? };
        if info.has_de() { write!(w, " de")? };
        if info.has_pse() { write!(w, " pse")? };
        if info.has_tsc() { write!(w, " tsc")? };
        if info.has_msr() { write!(w, " msr")? };
        if info.has_pae() { write!(w, " pae")? };
        if info.has_mce() { write!(w, " mce")? };

        if info.has_cmpxchg8b() { write!(w, " cx8")? };
        if info.has_apic() { write!(w, " apic")? };
        if info.has_sysenter_sysexit() { write!(w, " sep")? };
        if info.has_mtrr() { write!(w, " mtrr")? };
        if info.has_pge() { write!(w, " pge")? };
        if info.has_mca() { write!(w, " mca")? };
        if info.has_cmov() { write!(w, " cmov")? };
        if info.has_pat() { write!(w, " pat")? };

        if info.has_pse36() { write!(w, " pse36")? };
        if info.has_psn() { write!(w, " psn")? };
        if info.has_clflush() { write!(w, " clflush")? };
        if info.has_ds() { write!(w, " ds")? };
        if info.has_acpi() { write!(w, " acpi")? };
        if info.has_mmx() { write!(w, " mmx")? };
        if info.has_fxsave_fxstor() { write!(w, " fxsr")? };
        if info.has_sse() { write!(w, " sse")? };

        if info.has_sse2() { write!(w, " sse2")? };
        if info.has_ss() { write!(w, " ss")? };
        if info.has_htt() { write!(w, " ht")? };
        if info.has_tm() { write!(w, " tm")? };
        if info.has_pbe() { write!(w, " pbe")? };

        if info.has_sse3() { write!(w, " sse3")? };
        if info.has_pclmulqdq() { write!(w, " pclmulqdq")? };
        if info.has_ds_area() { write!(w, " dtes64")? };
        if info.has_monitor_mwait() { write!(w, " monitor")? };
        if info.has_cpl() { write!(w, " ds_cpl")? };
        if info.has_vmx() { write!(w, " vmx")? };
        if info.has_smx() { write!(w, " smx")? };
        if info.has_eist() { write!(w, " est")? };

        if info.has_tm2() { write!(w, " tm2")? };
        if info.has_ssse3() { write!(w, " ssse3")? };
        if info.has_cnxtid() { write!(w, " cnxtid")? };
        if info.has_fma() { write!(w, " fma")? };
        if info.has_cmpxchg16b() { write!(w, " cx16")? };
        if info.has_pdcm() { write!(w, " pdcm")? };
        if info.has_pcid() { write!(w, " pcid")? };
        if info.has_dca() { write!(w, " dca")? };

        if info.has_sse41() { write!(w, " sse4_1")? };
        if info.has_sse42() { write!(w, " sse4_2")? };
        if info.has_x2apic() { write!(w, " x2apic")? };
        if info.has_movbe() { write!(w, " movbe")? };
        if info.has_popcnt() { write!(w, " popcnt")? };
        if info.has_tsc_deadline() { write!(w, " tsc_deadline_timer")? };
        if info.has_aesni() { write!(w, " aes")? };
        if info.has_xsave() { write!(w, " xsave")? };

        if info.has_oxsave() { write!(w, " xsaveopt")? };
        if info.has_avx() { write!(w, " avx")? };
        if info.has_f16c() { write!(w, " f16c")? };
        if info.has_rdrand() { write!(w, " rdrand")? };
    }

    if let Some(info) = cpuid.get_extended_function_info() {
        if info.has_64bit_mode() { write!(w, " lm")? };
        if info.has_rdtscp() { write!(w, " rdtscp")? };
        if info.has_1gib_pages() { write!(w, " pdpe1gb")? };
        if info.has_execute_disable() { write!(w, " nx")? };
        if info.has_syscall_sysret() { write!(w, " syscall")? };
        if info.has_prefetchw() { write!(w, " prefetchw")? };
        if info.has_lzcnt() { write!(w, " lzcnt")? };
        if info.has_lahf_sahf() { write!(w, " lahf_lm")? };
        if info.has_invariant_tsc() { write!(w, " constant_tsc")? };
    }

    if let Some(info) = cpuid.get_extended_feature_info() {
        if info.has_fsgsbase() { write!(w, " fsgsbase")? };
        if info.has_tsc_adjust_msr() { write!(w, " tsc_adjust")? };
        if info.has_bmi1() { write!(w, " bmi1")? };
        if info.has_hle() { write!(w, " hle")? };
        if info.has_avx2() { write!(w, " avx2")? };
        if info.has_smep() { write!(w, " smep")? };
        if info.has_bmi2() { write!(w, " bmi2")? };
        if info.has_rep_movsb_stosb() { write!(w, " erms")? };
        if info.has_invpcid() { write!(w, " invpcid")? };
        if info.has_rtm() { write!(w, " rtm")? };
        if info.has_qm() { write!(w, " qm")? };
        if info.has_fpu_cs_ds_deprecated() { write!(w, " fpu_seg")? };
        if info.has_mpx() { write!(w, " mpx")? };
    }

    write!(w, "\n")?;

    Ok(())
}
