// TODO: Cleanup and make correct and safe

static mut NEXT: u64 = 0;

/// Generate pseudo random number
pub fn rand() -> usize {
    unsafe {
        NEXT = NEXT * 1103515245 + 12345;
        (NEXT / 65536) as usize
    }
}
