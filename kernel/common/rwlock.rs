use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

use common::mutex::Mutex;

use scheduler::context::context_switch;

struct RwLockInner {
    writer: bool,
    readers: usize
}

pub struct RwLock<T> {
    inner: Mutex<RwLockInner>,
    value: UnsafeCell<T>
}

impl<T> RwLock<T> {
    /// Create a new mutex with value `value`.
    pub fn new(value: T) -> Self {
        RwLock {
            inner: Mutex::new(RwLockInner {
                writer: false,
                readers: 0
            }),
            value: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    /// Lock the mutex
    pub fn read(&self) -> RwLockReadGuard<T> {
        loop {
            {
                let mut inner = self.inner.lock();
                if inner.writer == false {
                    inner.readers += 1;
                    break;
                }
            }
            unsafe { context_switch(false) };
        }
        RwLockReadGuard::new(&self.inner, &self.value)
    }

    pub fn write(&self) -> RwLockWriteGuard<T> {
        loop {
            {
                let mut inner = self.inner.lock();
                if inner.writer == false && inner.readers == 0 {
                    inner.writer = true;
                    break;
                }
            }
            unsafe { context_switch(false) };
        }
        RwLockWriteGuard::new(&self.inner, &self.value)
    }
}

unsafe impl<T: ?Sized + Send> Send for RwLock<T> { }

unsafe impl<T: ?Sized + Send> Sync for RwLock<T> { }

/// A read guard (returned by .read())
pub struct RwLockReadGuard<'a, T: ?Sized + 'a> {
    inner: &'a Mutex<RwLockInner>,
    data: &'a UnsafeCell<T>,
}

impl<'mutex, T: ?Sized> RwLockReadGuard<'mutex, T> {
    fn new(inner: &'mutex Mutex<RwLockInner>, data: &'mutex UnsafeCell<T>) -> Self {
        RwLockReadGuard {
            inner: inner,
            data: data,
        }
    }
}

impl<'mutex, T: ?Sized> Deref for RwLockReadGuard<'mutex, T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for RwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        let mut inner = self.inner.lock();
        inner.readers -= 1;
    }
}


/// A write guard (returned by .write())
pub struct RwLockWriteGuard<'a, T: ?Sized + 'a> {
    inner: &'a Mutex<RwLockInner>,
    data: &'a UnsafeCell<T>,
}

impl<'mutex, T: ?Sized> RwLockWriteGuard<'mutex, T> {
    fn new(inner: &'mutex Mutex<RwLockInner>, data: &'mutex UnsafeCell<T>) -> Self {
        RwLockWriteGuard {
            inner: inner,
            data: data,
        }
    }
}

impl<'mutex, T: ?Sized> Deref for RwLockWriteGuard<'mutex, T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.data.get() }
    }
}

impl<'mutex, T: ?Sized> DerefMut for RwLockWriteGuard<'mutex, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        let mut inner = self.inner.lock();
        inner.writer = false;
    }
}
