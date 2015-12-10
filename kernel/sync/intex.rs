use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut, Drop};

pub static mut intex_count: usize = 0;

/// An Intex, interrupt exclusion during value usage
pub struct Intex<T: ?Sized> {
    value: UnsafeCell<T>,
}

impl Intex<()> {
    pub fn static_lock() -> StaticIntexGuard {
        StaticIntexGuard
    }
}

impl<T> Intex<T> {
    /// Create a new Intex with value `value`.
    pub fn new(value: T) -> Self {
        Intex { value: UnsafeCell::new(value) }
    }
}

impl<T: ?Sized> Intex<T> {
    /// Lock the Intex
    pub fn lock(&self) -> IntexGuard<T> {
        IntexGuard::new(&self.value)
    }
}

unsafe impl<T: ?Sized + Send> Send for Intex<T> { }

unsafe impl<T: ?Sized + Send> Sync for Intex<T> { }

/// A Intex guard (returned by .lock())
pub struct IntexGuard<'a, T: ?Sized + 'a> {
    inner: StaticIntexGuard,
    data: &'a UnsafeCell<T>,
}

impl<'intex, T: ?Sized> IntexGuard<'intex, T> {
    fn new(data: &'intex UnsafeCell<T>) -> Self {
        IntexGuard {
            inner: StaticIntexGuard::new(),
            data: data,
        }
    }
}

impl<'intex, T: ?Sized> Deref for IntexGuard<'intex, T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.data.get() }
    }
}

impl<'intex, T: ?Sized> DerefMut for IntexGuard<'intex, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.data.get() }
    }
}

/// A Static Intex guard (returned by .static_lock())
pub struct StaticIntexGuard;

impl StaticIntexGuard {
    fn new() -> Self {
        unsafe {
            asm!("cli");
            intex_count += 1;
        }
        StaticIntexGuard
    }
}

impl Drop for StaticIntexGuard {
    fn drop(&mut self) {
        unsafe {
            intex_count -= 1;
            if intex_count == 0 {
                //asm!("sti");
            }
        }
    }
}
