use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

use sync::Mutex;

use thread;

struct RwLockInner {
    writer: bool,
    readers: usize,
}

pub struct RwLock<T: ?Sized> {
    inner: Mutex<RwLockInner>,
    value: UnsafeCell<T>,
}

impl<T> RwLock<T> {
    /// Create a new mutex with value `value`.
    pub fn new(value: T) -> Self {
        RwLock {
            inner: Mutex::new(RwLockInner {
                writer: false,
                readers: 0,
            }),
            value: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    /// Lock for read
    pub fn read(&self) -> RwLockReadGuard<T> {
        loop {
            {
                let mut inner = self.inner.lock().unwrap();
                if inner.writer == false {
                    inner.readers += 1;
                    break;
                }
            }
            thread::yield_now();
        }
        RwLockReadGuard::new(&self.inner, &self.value)
    }

    /// Lock for write
    pub fn write(&self) -> RwLockWriteGuard<T> {
        loop {
            {
                let mut inner = self.inner.lock().unwrap();
                if inner.writer == false && inner.readers == 0 {
                    inner.writer = true;
                    break;
                }
            }
            thread::yield_now();
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

impl<'rwlock, T: ?Sized> RwLockReadGuard<'rwlock, T> {
    fn new(inner: &'rwlock Mutex<RwLockInner>, data: &'rwlock UnsafeCell<T>) -> Self {
        RwLockReadGuard {
            inner: inner,
            data: data,
        }
    }
}

impl<'rwlock, T: ?Sized> Deref for RwLockReadGuard<'rwlock, T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for RwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.readers -= 1;
    }
}


/// A write guard (returned by .write())
pub struct RwLockWriteGuard<'a, T: ?Sized + 'a> {
    inner: &'a Mutex<RwLockInner>,
    data: &'a UnsafeCell<T>,
}

impl<'rwlock, T: ?Sized> RwLockWriteGuard<'rwlock, T> {
    fn new(inner: &'rwlock Mutex<RwLockInner>, data: &'rwlock UnsafeCell<T>) -> Self {
        RwLockWriteGuard {
            inner: inner,
            data: data,
        }
    }
}

impl<'rwlock, T: ?Sized> Deref for RwLockWriteGuard<'rwlock, T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.data.get() }
    }
}

impl<'rwlock, T: ?Sized> DerefMut for RwLockWriteGuard<'rwlock, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.writer = false;
    }
}
