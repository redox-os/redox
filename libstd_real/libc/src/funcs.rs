use super::{c_char, size_t};

pub unsafe extern fn strlen(ptr: *const c_char) -> size_t {
    let mut i: size_t = 0;
    while *ptr.offset(i as isize) != 0 {
        i += 1;
    }
    i
}

pub unsafe extern fn random() -> u64 {
    let rand;
    asm!("rdrand rax"
        : "={rax}"(rand)
        :
        :
        : "intel", "volatile");
    rand
}
