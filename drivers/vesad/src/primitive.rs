#[cfg(target_arch = "x86_64")]
#[allow(unused_assignments)]
#[inline(always)]
#[cold]
pub unsafe fn fast_copy(dst: *mut u8, src: *const u8, len: usize) {
    asm!("cld
        rep movsb"
        :
        : "{rdi}"(dst as usize), "{rsi}"(src as usize), "{rcx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}

#[cfg(target_arch = "x86_64")]
#[allow(unused_assignments)]
#[inline(always)]
#[cold]
pub unsafe fn fast_copy64(dst: *mut u64, src: *const u64, len: usize) {
    asm!("cld
        rep movsq"
        :
        : "{rdi}"(dst as usize), "{rsi}"(src as usize), "{rcx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}

#[cfg(target_arch = "x86_64")]
#[allow(unused_assignments)]
#[inline(always)]
#[cold]
pub unsafe fn fast_set32(dst: *mut u32, src: u32, len: usize) {
    asm!("cld
        rep stosd"
        :
        : "{rdi}"(dst as usize), "{eax}"(src), "{rcx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}

#[cfg(target_arch = "x86_64")]
#[allow(unused_assignments)]
#[inline(always)]
#[cold]
pub unsafe fn fast_set64(dst: *mut u64, src: u64, len: usize) {
    asm!("cld
        rep stosq"
        :
        : "{rdi}"(dst as usize), "{rax}"(src), "{rcx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}
