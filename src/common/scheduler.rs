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

pub unsafe fn end_no_ints(reenable: bool) {
    if reenable {
        asm!("sti"
            :
            :
            :
            : "intel", "volatile");
    }
}

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

pub unsafe fn end_ints(disable: bool) {
    if disable {
        asm!("cli"
            :
            :
            :
            : "intel", "volatile");
    }
}
