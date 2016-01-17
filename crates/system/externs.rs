use core::ptr;

#[lang="stack_exhausted"]
extern "C" fn stack_exhausted() {}

#[lang="eh_personality"]
extern "C" fn eh_personality() {}

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
/// 64 bit remainder on 32 bit arch
pub extern "C" fn __umoddi3(mut a: u64, mut b: u64) -> u64 {
    let mut hig = a >> 32; // The first 32 bits of a
    let mut d = 1;

    if hig >= b {
        hig /= b;
        a -= (hig * b) << 32;
    }

    while b > 0 && b < a {
        b *= 2;
        d *= 2;
    }

    loop {
        if a >= b {
            a -= b;
        }
        b >>= 1;
        d >>= 1;

        if d == 0 {
            break;
        }
    }

    a
}

#[no_mangle]
#[cfg(target_arch = "x86")]
/// 64 bit division on 32 bit arch
pub extern "C" fn __udivdi3(mut a: u64, mut b: u64) -> u64 {
    let mut res = 0;
    let mut hig = a >> 32; // The first 32 bits of a
    let mut d = 1;

    if hig >= b {
        hig /= b;
        res = hig << 32;
        a -= (hig * b) << 32;
    }

    while b > 0 && b < a {
        b *= 2;
        d *= 2;
    }

    loop {
        if a >= b {
            a -= b;
            res += d;
        }
        b >>= 1;
        d >>= 1;

        if d == 0 {
            break;
        }
    }

    res
}

#[no_mangle]
#[cfg(target_arch = "x86")]
/// 64 bit division and rem on 32 bit arch
pub extern "C" fn __udivremi3(mut a: u64, mut b: u64) -> (u64, u64) {
    let mut res = 0;
    let mut hig = a >> 32; // The first 32 bits of a
    let mut d = 1;

    if hig >= b {
        hig /= b;
        res = hig << 32;
        a -= (hig * b) << 32;
    }

    while b > 0 && b < a {
        b *= 2;
        d *= 2;
    }

    loop {
        if a >= b {
            a -= b;
            res += d;
        }
        b >>= 1;
        d >>= 1;

        if d == 0 {
            break;
        }
    }

    (res, a)
}
