#[naked]
pub unsafe extern fn syscall() {
    #[inline(never)]
    unsafe fn inner() {
        extern {
            fn syscall(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> usize;
        }

        let mut a;
        {
            let b;
            let c;
            let d;
            let e;
            let f;
            asm!("" : "={rax}"(a), "={rbx}"(b), "={rcx}"(c), "={rdx}"(d), "={rsi}"(e), "={rdi}"(f)
                : : : "intel", "volatile");

            a = syscall(a, b, c, d, e, f);
        }

        asm!("" : : "{rax}"(a) : : "intel", "volatile");
    }

    asm!("push fs
        push rax
        mov rax, 0x18
        mov fs, ax
        pop rax"
        : : : : "intel", "volatile");

    inner();

    // Interrupt return
    asm!("pop fs
        iretq"
        : : : : "intel", "volatile");
}
