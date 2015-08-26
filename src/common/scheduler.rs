pub fn sched_yield(){
    unsafe {
        asm!("int 0x80"
            :
            : "{eax}"(3)
            :
            : "intel");
    }
}

pub fn sched_exit() {
    // TODO: Wrap in no ints?
    unsafe {
        asm!("int 0x80"
            :
            : "{eax}"(4)
            :
            : "intel");
    }
}

pub unsafe fn start_no_ints() -> bool {
    let flags: u32;
    asm!("pushfd
        cli
        pop eax"
        : "={eax}"(flags) : : : "intel");
    return flags & (1 << 9) == (1 << 9);
}

pub unsafe fn end_no_ints(reenable: bool) {
    if reenable {
        asm!("sti");
    }
}
