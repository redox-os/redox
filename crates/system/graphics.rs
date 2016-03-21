#[cfg(target_arch = "x86")]
#[allow(unused_assignments)]
#[inline(always)]
pub unsafe fn fast_copy(mut dst: *mut u32, mut src: *const u32, mut len: usize) {
    asm!("cld
        rep movsb"
        : "={edi}"(dst), "={esi}"(src), "={ecx}"(len)
        : "{edi}"(dst as usize), "{esi}"(src as usize), "{ecx}"(len * 4)
        : "memory"
        : "intel", "volatile");
}

#[cfg(target_arch = "x86")]
#[allow(unused_assignments)]
#[inline(always)]
pub unsafe fn fast_set(mut dst: *mut u32, mut src: u32, mut len: usize) {
    asm!("cld
        rep stosd"
        : "={edi}"(dst), "={eax}"(src), "={ecx}"(len)
        : "{edi}"(dst as usize), "{eax}"(src), "{ecx}"(len)
        : "memory"
        : "intel", "volatile");
}

#[cfg(target_arch = "x86_64")]
#[allow(unused_assignments)]
#[inline(always)]
pub unsafe fn fast_copy(mut dst: *mut u32, mut src: *const u32, mut len: usize) {
    asm!("cld
        rep movsb"
        : "={rdi}"(dst), "={rsi}"(src), "={rcx}"(len)
        : "{rdi}"(dst as usize), "{rsi}"(src as usize), "{rcx}"(len * 4)
        : "memory"
        : "intel", "volatile");
}

#[cfg(target_arch = "x86_64")]
#[allow(unused_assignments)]
#[inline(always)]
pub unsafe fn fast_set(mut dst: *mut u32, mut src: u32, mut len: usize) {
    asm!("cld
        rep stosd"
        : "={rdi}"(dst), "={eax}"(src), "={rcx}"(len)
        : "{rdi}"(dst as usize), "{eax}"(src), "{rcx}"(len)
        : "memory"
        : "intel", "volatile");
}
