//Left uninitialized (because of static mut initialization properties)
static mut next: u64 = 1;

pub unsafe fn rand() -> usize {
    next = next * 1103515245 + 12345;
    return (next / 65536) as usize;
}

pub unsafe fn srand(seed: usize){
    next = seed as u64;
}