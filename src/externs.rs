use core::fmt;
use core::ptr;
use core::result;

use common::debug::*;
use common::memory::*;

use syscall::handle::do_sys_exit;

struct DebugStream;

impl fmt::Write for DebugStream {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        d(s);

        result::Result::Ok(())
    }
}

#[lang="panic_fmt"]
pub extern fn panic_fmt(args: fmt::Arguments, file: &'static str, line: u32) -> ! {
    d(file);
    d(":");
    dd(line as usize);
    d(": ");
    fmt::write(&mut DebugStream, args);
    dl();

    unsafe {
        do_sys_exit(-1);
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

#[allow(unused_variables)]
#[no_mangle]
pub extern fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    unsafe { alloc(size) as *mut u8 }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn __rust_deallocate(ptr: *mut u8, old_size: usize, align: usize) {
    unsafe { unalloc(ptr as usize) }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn __rust_reallocate(ptr: *mut u8, old_size: usize, size: usize,
                                align: usize) -> *mut u8 {
    unsafe { realloc(ptr as usize, size) as *mut u8 }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn __rust_reallocate_inplace(ptr: *mut u8, old_size: usize,
                                        size: usize, align: usize) -> usize {
    unsafe { realloc_inplace(ptr as usize, size) }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn __rust_usable_size(size: usize, align: usize) -> usize {
    size
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *mut u8, b: *const u8, len: usize) -> isize {
    for i in 0..len {
        let c_a = ptr::read(a.offset(i as isize));
        let c_b = ptr::read(b.offset(i as isize));
        if c_a != c_b {
            return c_a as isize - c_b as isize;
        }
    }
    return 0;
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
