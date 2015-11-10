pub use self::context::Context;
pub use self::regs::Regs;
pub use self::tss::TSS;

/// Context
pub mod context;
pub mod regs;
pub mod tss;

#[cfg(target_arch = "x86")]
pub unsafe fn start_no_ints() -> bool {
    let flags: u32;
    asm!("pushfd
        cli
        pop eax"
        : "={eax}"(flags)
        :
        : "memory"
        : "intel", "volatile");
    flags & (1 << 9) == (1 << 9)
}

#[cfg(target_arch = "x86")]
pub unsafe fn end_no_ints(reenable: bool) {
    if reenable {
        asm!("sti"
            :
            :
            :
            : "intel", "volatile");
    }
}

#[cfg(target_arch = "x86")]
pub unsafe fn start_ints() -> bool {
    let flags: u32;
    asm!("pushfd
        sti
        pop eax"
        : "={eax}"(flags)
        :
        : "memory"
        : "intel", "volatile");
    flags & (1 << 9) != (1 << 9)
}

#[cfg(target_arch = "x86")]
pub unsafe fn end_ints(disable: bool) {
    if disable {
        asm!("cli"
            :
            :
            :
            : "intel", "volatile");
    }
}

#[cfg(target_arch = "x86_64")]
pub unsafe fn start_no_ints() -> bool {
    let flags: u64;
    asm!("pushfq
        cli
        pop rax"
        : "={rax}"(flags)
        :
        : "memory"
        : "intel", "volatile");
    flags & (1 << 9) == (1 << 9)
}

#[cfg(target_arch = "x86_64")]
pub unsafe fn end_no_ints(reenable: bool) {
    if reenable {
        asm!("sti"
            :
            :
            :
            : "intel", "volatile");
    }
}

#[cfg(target_arch = "x86_64")]
pub unsafe fn start_ints() -> bool {
    let flags: u64;
    asm!("pushfq
        sti
        pop rax"
        : "={rax}"(flags)
        :
        : "memory"
        : "intel", "volatile");
    flags & (1 << 9) != (1 << 9)
}

#[cfg(target_arch = "x86_64")]
pub unsafe fn end_ints(disable: bool) {
    if disable {
        asm!("cli"
            :
            :
            :
            : "intel", "volatile");
    }
}
