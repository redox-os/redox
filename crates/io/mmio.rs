use core::intrinsics::{volatile_load, volatile_store};

use super::io::Io;

#[repr(packed)]
pub struct Mmio<T> {
    value: T,
}

impl <T> Io<T> for Mmio<T> {
    fn read(&self) -> T {
        unsafe { volatile_load(&self.value) }
    }

    fn write(&mut self, value: T) {
        unsafe { volatile_store(&mut self.value, value) };
    }
}
