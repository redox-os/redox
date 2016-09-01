#[naked]
pub unsafe extern fn syscall() {
    extern {
        fn syscall(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> usize;
    }

    let a;
    let b;
    let c;
    let d;
    let e;
    let f;
    asm!("" : "={rax}"(a), "={rbx}"(b), "={rcx}"(c), "={rdx}"(d), "={rsi}"(e), "={rdi}"(f)
        : : : "intel", "volatile");

    let a = syscall(a, b, c, d, e, f);

    asm!("" : : "{rax}"(a) : : "intel", "volatile");

    // Pop scratch registers, error code, and return
    asm!("iretq" : : : : "intel", "volatile");
}
