const NEXT: *mut u64 = 0x200010 as *mut u64;

pub fn rand() -> usize {
    unsafe {
        (*NEXT) = (*NEXT) * 1103515245 + 12345;
        return ((*NEXT) / 65536) as usize;
    }
}

pub fn srand(seed: usize){
    unsafe {
        (*NEXT) = seed as u64;
    }
}
