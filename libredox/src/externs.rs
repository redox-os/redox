use core::fmt;
use core::fmt::Write;
use core::result;

use syscall::*;

pub struct DebugStream;

impl Write for DebugStream {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            unsafe { sys_debug(b) };
        }

        result::Result::Ok(())
    }
}

#[lang="panic_fmt"]
#[allow(unused_must_use)]
pub extern fn panic_impl(args: fmt::Arguments, file: &'static str, line: u32) -> ! {
    let mut stream = DebugStream;
    fmt::write(&mut stream, args);
    fmt::write(&mut stream, format_args!(" in {}:{}\n", file, line));

    unsafe {
        sys_exit(-1);
        loop {
            asm!("sti");
            asm!("hlt");
        }
    }
}

#[lang="stack_exhausted"]
extern "C" fn stack_exhausted() {

}

#[lang="eh_personality"]
extern "C" fn eh_personality() {

}

#[allocator]
#[allow(unused_variables)]
#[no_mangle]
pub extern fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    unsafe { sys_alloc(size) as *mut u8 }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn __rust_deallocate(ptr: *mut u8, old_size: usize, align: usize) {
    unsafe { sys_unalloc(ptr as usize) }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn __rust_reallocate(ptr: *mut u8, old_size: usize, size: usize,
                                align: usize) -> *mut u8 {
    unsafe { sys_realloc(ptr as usize, size) as *mut u8 }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn __rust_reallocate_inplace(ptr: *mut u8, old_size: usize,
                                        size: usize, align: usize) -> usize {
    unsafe { sys_realloc_inplace(ptr as usize, size) }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn __rust_usable_size(size: usize, align: usize) -> usize {
    size
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "C" fn memcmp(a: *const i8, b: *const i8, len: usize) -> i32 {
    let ret;
    asm!("cld
        repne cmpsb
        xor eax, eax
        mov al, [edi]
        xor ecx, ecx
        mov cl, [esi]
        sub eax, ecx"
        : "={eax}"(ret)
        : "{edi}"(a), "{esi}"(b), "{ecx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
    ret
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, len: usize) {
    if src < dst {
        asm!("std
            rep movsb"
            :
            : "{edi}"(dst.offset(len as isize - 1)), "{esi}"(src.offset(len as isize - 1)), "{ecx}"(len)
            : "cc", "memory"
            : "intel", "volatile");
    } else {
        asm!("cld
            rep movsb"
            :
            : "{edi}"(dst), "{esi}"(src), "{ecx}"(len)
            : "cc", "memory"
            : "intel", "volatile");
    }
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, len: usize) {
    asm!("cld
        rep movsb"
        :
        : "{edi}"(dst), "{esi}"(src), "{ecx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "C" fn memset(dst: *mut u8, c: i32, len: usize) {
    asm!("cld
        rep stosb"
        :
        : "{eax}"(c), "{edi}"(dst), "{ecx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn memcmp(a: *const i8, b: *const i8, len: usize) -> i32 {
    let ret;
    asm!("cld
        repne cmpsb
        xor rax, rax
        mov al, [rdi]
        xor rcx, rcx
        mov cl, [rsi]
        sub rax, rcx"
        : "={rax}"(ret)
        : "{rdi}"(a), "{rsi}"(b), "{rcx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
    ret
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, len: usize) {
    if src < dst {
        asm!("std
            rep movsb"
            :
            : "{rdi}"(dst.offset(len as isize - 1)), "{rsi}"(src.offset(len as isize - 1)), "{rcx}"(len)
            : "cc", "memory"
            : "intel", "volatile");
    } else {
        asm!("cld
            rep movsb"
            :
            : "{rdi}"(dst), "{rsi}"(src), "{rcx}"(len)
            : "cc", "memory"
            : "intel", "volatile");
    }
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, len: usize) {
    asm!("cld
        rep movsb"
        :
        : "{rdi}"(dst), "{rsi}"(src), "{rcx}"(len)
        : "cc", "memory"
        : "intel", "volatile");
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn memset(dst: *mut u8, c: i32, len: usize) {
    asm!("cld
        rep stosb"
        :
        : "{rax}"(c), "{rdi}"(dst), "{rcx}"(len)
        : "cc", "memory"
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
