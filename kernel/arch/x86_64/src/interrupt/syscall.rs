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

    // Push scratch registers, minus rax for the return value
    asm!("push rcx
        push rdx
        push rdi
        push rsi
        push r8
        push r9
        push r10
        push r11
        push fs
        mov r11, 0x18
        mov fs, r11"
        : : : : "intel", "volatile");

    inner();

    // Interrupt return
    asm!("pop fs
        pop r11
        pop r10
        pop r9
        pop r8
        pop rsi
        pop rdi
        pop rdx
        pop rcx
        iretq"
        : : : : "intel", "volatile");
}

#[naked]
pub unsafe extern fn clone_ret() -> usize {
    asm!("pop rbp"
        : : : : "intel", "volatile");
        0
}
