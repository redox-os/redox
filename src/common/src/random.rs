const NEXT: *mut u64 = 0x200010 as *mut u64;

/// Generate pseudo random number
pub fn rand() -> usize {
    unsafe {
        (*NEXT) = (*NEXT) * 1103515245 + 12345;
        ((*NEXT) / 65536) as usize
    }
}

/// Generate pseudo random number via seed
pub fn srand(seed: usize) {
    unsafe {
        (*NEXT) = seed as u64;
    }
}
