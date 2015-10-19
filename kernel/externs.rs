use core::ptr;

#[lang="stack_exhausted"]
extern "C" fn stack_exhausted() {

}

#[lang="eh_personality"]
extern "C" fn eh_personality() {

}

#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *mut i8, b: *const i8, len: usize) -> i32 {
    for i in 0..len {
        let c_a = ptr::read(a.offset(i as isize));
        let c_b = ptr::read(b.offset(i as isize));
        if c_a != c_b {
            return c_a as i32 - c_b as i32;
        }
    }
    return 0;
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, len: usize) {
    if src < dst {
        asm!("pushfd
            std
            rep movsb
            popfd"
            :
            : "{edi}"(dst.offset(len as isize - 1)), "{esi}"(src.offset(len as isize - 1)), "{ecx}"(len)
            : "{edi}", "{esi}", "{ecx}", "memory"
            : "intel", "volatile");
    } else {
        asm!("pushfd
            cld
            rep movsb
            popfd"
            :
            : "{edi}"(dst), "{esi}"(src), "{ecx}"(len)
            : "{edi}", "{esi}", "{ecx}", "memory"
            : "intel", "volatile");
    }
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, len: usize) {
    asm!("pushfd
        cld
        rep movsb
        popfd"
        :
        : "{edi}"(dst), "{esi}"(src), "{ecx}"(len)
        : "{edi}", "{esi}", "{ecx}", "memory"
        : "intel", "volatile");
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "C" fn memset(dst: *mut u8, c: i32, len: usize) {
    asm!("pushfd
        cld
        rep stosb
        popfd"
        :
        : "{eax}"(c), "{edi}"(dst), "{ecx}"(len)
        : "{edi}", "{ecx}", "memory"
        : "intel", "volatile");
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, len: usize) {
    if src < dst {
        asm!("pushfq
            std
            rep movsb
            popfq"
            :
            : "{rdi}"(dst.offset(len as isize - 1)), "{rsi}"(src.offset(len as isize - 1)), "{rcx}"(len)
            : "{rdi}", "{rsi}", "{rcx}", "memory"
            : "intel", "volatile");
    } else {
        asm!("pushfq
            cld
            rep movsb
            popfq"
            :
            : "{rdi}"(dst), "{rsi}"(src), "{rcx}"(len)
            : "{rdi}", "{rsi}", "{rcx}", "memory"
            : "intel", "volatile");
    }
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, len: usize) {
    asm!("pushfq
        cld
        rep movsb
        popfq"
        :
        : "{rdi}"(dst), "{rsi}"(src), "{rcx}"(len)
        : "{rdi}", "{rsi}", "{rcx}", "memory"
        : "intel", "volatile");
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn memset(dst: *mut u8, c: i32, len: usize) {
    asm!("pushfq
        cld
        rep stosb
        popfq"
        :
        : "{rax}"(c), "{rdi}"(dst), "{rcx}"(len)
        : "{rdi}", "{rcx}", "memory"
        : "intel", "volatile");
}


#[no_mangle]
#[cfg(target_arch = "x86")]
//TODO Make this better
/// 64 bit remainder on 32 bit arch
pub extern "C" fn __umoddi3(a: u64, b: u64) -> u64 {
    if b == 0 {
        return 0;
    }

    let mut rem = a;
    while rem >= b {
        rem -= b;
    }
    rem
}

#[no_mangle]
#[cfg(target_arch = "x86")]
//TODO Make this better
/// 64 bit division on 32 bit arch
pub extern "C" fn __udivdi3(a: u64, b: u64) -> u64 {
    if b == 0 {
        return 0;
    }

    let mut quot = 0;
    let mut rem = a;
    while rem >= b {
        rem -= b;
        quot += 1;
    }
    quot
}
