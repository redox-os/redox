use core::intrinsics::{volatile_load, volatile_store};
use core::mem::uninitialized;
use core::ops::{BitAnd, BitOr, Not};

use super::io::Io;

#[repr(packed)]
pub struct Mmio<T> {
    value: T,
}

impl<T> Mmio<T> {
    /// Create a new Mmio without initializing
    pub fn new() -> Self {
        Mmio {
            value: unsafe { uninitialized() }
        }
    }
}

impl<T> Io for Mmio<T> where T: Copy + PartialEq + BitAnd<Output = T> + BitOr<Output = T> + Not<Output = T> {
    type Value = T;

    fn read(&self) -> T {
        unsafe { volatile_load(&self.value) }
    }

    fn write(&mut self, value: T) {
        unsafe { volatile_store(&mut self.value, value) };
    }
}
