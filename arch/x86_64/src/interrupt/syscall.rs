#[naked]
pub unsafe extern fn syscall() {
    #[inline(never)]
    unsafe fn inner() {
        extern {
            fn syscall(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize, stack: usize) -> usize;
        }

        let mut a;
        {
            let b;
            let c;
            let d;
            let e;
            let f;
            let stack;
            asm!("" : "={rax}"(a), "={rbx}"(b), "={rcx}"(c), "={rdx}"(d), "={rsi}"(e), "={rdi}"(f), "={rbp}"(stack)
                : : : "intel", "volatile");

            a = syscall(a, b, c, d, e, f, stack);
        }

        asm!("" : : "{rax}"(a) : : "intel", "volatile");
    }

    asm!("push r15
        push fs
        mov r15, 0x18
        mov fs, r15"
        : : : : "intel", "volatile");

    inner();

    // Interrupt return
    asm!("pop fs
        pop r15
        iretq"
        : : : : "intel", "volatile");
}

#[naked]
pub unsafe extern fn clone_ret() -> usize {
    asm!("pop rbp"
        : : : : "intel", "volatile");
        0
}
