extern crate io;

pub use self::io::*;

use arch::memory::LOGICAL_OFFSET;

/// A wrapper for physical addresses
/// T is the size the physical address should be stored in
#[repr(packed)]
pub struct PhysAddr<I: Io> {
    inner: I
}

impl<I: Io> PhysAddr<I> {
    pub fn new(inner: I) -> PhysAddr<I> {
        PhysAddr {
            inner: inner
        }
    }
}

impl<I: Io<Value=u32>> PhysAddr<I> {
    /// Write a value that may be logical
    pub fn write(&mut self, mut value: u32) {
        if value >= LOGICAL_OFFSET as u32 {
            value -= LOGICAL_OFFSET as u32;
        }
        self.inner.write(value);
    }
}
