extern crate io;

pub use self::io::*;

use arch::memory::LOGICAL_OFFSET;
use core::marker::PhantomData;

/// A wrapper for physical addresses
/// T is the size the physical address should be stored in
#[repr(packed)]
pub struct PhysAddr<T, I: Io<T>> {
    inner: I,
    value: PhantomData<T>
}

impl<T, I: Io<T>> PhysAddr<T, I> {
    pub fn new(inner: I) -> PhysAddr<T, I> {
        PhysAddr {
            inner: inner,
            value: PhantomData,
        }
    }
}

impl<I: Io<u32>> PhysAddr<u32, I> {
    /// Read the current value
    /// Unsafe because translation is unknown
    pub unsafe fn read(&self) -> u32 {
        self.inner.read()
    }

    /// Write a value that may be logical
    pub fn write(&mut self, mut value: u32) {
        if value >= LOGICAL_OFFSET as u32 {
            value -= LOGICAL_OFFSET as u32;
        }
        self.inner.write(value);
    }
}

impl<I: Io<u64>> PhysAddr<u64, I> {
    /// Read the current value
    /// Unsafe because translation is unknown
    pub unsafe fn read(&self) -> u64 {
        self.inner.read()
    }

    /// Write a value that may be logical
    pub fn write(&mut self, mut value: u64) {
        if value >= LOGICAL_OFFSET as u64 {
            value -= LOGICAL_OFFSET as u64;
        }
        self.inner.write(value);
    }
}
