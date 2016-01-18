pub use self::unix::*;
pub use self::redox::*;

// Unix compatible
pub mod unix;

// Redox special
pub mod redox;

#[cold]
#[inline(never)]
#[cfg(target_arch = "x86")]
pub unsafe fn syscall(mut a: usize, b: usize, c: usize, d: usize) -> usize {
    asm!("int 0x80"
        : "={eax}"(a)
        : "{eax}"(a), "{ebx}"(b), "{ecx}"(c), "{edx}"(d)
        : "memory"
        : "intel", "volatile");

    a
}

#[cold]
#[inline(never)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn syscall(mut a: usize, b: usize, c: usize, d: usize) -> usize {
    asm!("int 0x80"
        : "={rax}"(a)
        : "{rax}"(a), "{rbx}"(b), "{rcx}"(c), "{rdx}"(d)
        : "memory"
        : "intel", "volatile");

    a
}
