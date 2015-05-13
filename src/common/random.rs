//Left uninitialized (because of static mut initialization properties)
static mut next: u64 = 1;

pub fn rand() -> usize {
    unsafe {
        next = next * 1103515245 + 12345;
        return (next / 65536) as usize;
    }
}

pub fn srand(seed: usize){
    unsafe {
        next = seed as u64;
    }
}