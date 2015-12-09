use core::intrinsics::{volatile_load, volatile_store};

pub struct Mmio<T> {
    address: *mut T,
}

impl<T> Mmio<T> {
    fn new(address: *mut T) -> Self {
        return Mmio { address: address };
    }

    unsafe fn read(&self) -> T {
        return volatile_load(self.address);
    }

    unsafe fn write(&mut self, value: T) {
        volatile_store(self.address, value);
    }
}
