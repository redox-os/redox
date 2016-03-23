#[lang="eh_personality"]
extern "C" fn eh_personality() {}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn _Unwind_Resume(_ex_obj: *mut ()) { }

/// Memcpy
///
/// Copy N bytes of memory from one location to another.
#[no_mangle]
pub unsafe extern fn memcpy(dest: *mut u8, src: *const u8,
                            n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.offset(i as isize) = *src.offset(i as isize);
        i += 1;
    }

    dest
}

/// Memmove
///
/// Copy N bytes of memory from src to dest. The memory areas may overlap.
#[no_mangle]
pub unsafe extern fn memmove(dest: *mut u8, src: *const u8,
                             n: usize) -> *mut u8 {
    if src < dest as *const u8 {
        let mut i = n;
        while i != 0 {
            i -= 1;
            *dest.offset(i as isize) = *src.offset(i as isize);
        }
    } else {
        let mut i = 0;
        while i < n {
            *dest.offset(i as isize) = *src.offset(i as isize);
            i += 1;
        }
    }

    dest
}

/// Memset
///
/// Fill a block of memory with a specified value.
#[no_mangle]
pub unsafe extern fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.offset(i as isize) = c as u8;
        i += 1;
    }

    s
}

/// Memcmp
///
/// Compare two blocks of memory.
#[no_mangle]
pub unsafe extern fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0;

    while i < n {
        let a = *s1.offset(i as isize);
        let b = *s2.offset(i as isize);
        if a != b {
            return a as i32 - b as i32
        }
        i += 1;
    }

    0
}

/// 64 bit remainder on 32 bit arch
#[no_mangle]
#[cfg(target_arch = "x86")]
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

/// 64 bit division on 32 bit arch
#[no_mangle]
#[cfg(target_arch = "x86")]
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
