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
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, len: usize) {
    if src < dst {
        for i_reverse in 0..len as isize {
            let i = len as isize - i_reverse - 1;
            ptr::write(dst.offset(i), ptr::read(src.offset(i)));
        }
    } else {
        for i in 0..len as isize {
            ptr::write(dst.offset(i), ptr::read(src.offset(i)));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, len: usize) {
    for i in 0..len as isize {
        ptr::write(dst.offset(i), ptr::read(src.offset(i)));
    }
}

#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut u8, c: i32, len: usize) {
    for i in 0..len as isize {
        ptr::write(dst.offset(i), c as u8);
    }
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
