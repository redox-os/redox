use core::intrinsics::{volatile_load, volatile_store};
use core::mem::uninitialized;

use super::io::Io;

#[repr(packed)]
pub struct Mmio<T> {
    value: T,
}

impl<T: Default> Mmio<T> {
    /// Create a new Mmio without initializing
    pub fn new() -> Self {
        Mmio {
            value: unsafe { uninitialized() }
        }
    }
}

impl<T> Io<T> for Mmio<T> {
    fn read(&self) -> T {
        unsafe { volatile_load(&self.value) }
    }

    fn write(&mut self, value: T) {
        unsafe { volatile_store(&mut self.value, value) };
    }
}
