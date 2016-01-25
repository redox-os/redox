pub unsafe fn syscall0(mut a: usize) -> usize {
    asm!("int 0x80"
        : "={eax}"(a)
        : "{eax}"(a)
        : "memory"
        : "intel", "volatile");

    a
}

pub unsafe fn syscall1(mut a: usize, b: usize) -> usize {
    asm!("int 0x80"
        : "={eax}"(a)
        : "{eax}"(a), "{ebx}"(b)
        : "memory"
        : "intel", "volatile");

    a
}

pub unsafe fn syscall2(mut a: usize, b: usize, c: usize) -> usize {
    asm!("int 0x80"
        : "={eax}"(a)
        : "{eax}"(a), "{ebx}"(b), "{ecx}"(c)
        : "memory"
        : "intel", "volatile");

    a
}

pub unsafe fn syscall3(mut a: usize, b: usize, c: usize, d: usize) -> usize {
    asm!("int 0x80"
        : "={eax}"(a)
        : "{eax}"(a), "{ebx}"(b), "{ecx}"(c), "{edx}"(d)
        : "memory"
        : "intel", "volatile");

    a
}

pub unsafe fn syscall4(mut a: usize, b: usize, c: usize, d: usize, e: usize) -> usize {
    asm!("int 0x80"
        : "={eax}"(a)
        : "{eax}"(a), "{ebx}"(b), "{ecx}"(c), "{edx}"(d), "{esi}"(e)
        : "memory"
        : "intel", "volatile");

    a
}

pub unsafe fn syscall5(mut a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> usize {
    asm!("int 0x80"
        : "={eax}"(a)
        : "{eax}"(a), "{ebx}"(b), "{ecx}"(c), "{edx}"(d), "{esi}"(e), "{edi}"(f)
        : "memory"
        : "intel", "volatile");

    a
}
