use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::usize;

use sync::Mutex;

use scheduler::context::context_switch;

struct RecursiveMutexInner {
    owner: usize,
    count: usize,
}

pub struct RecursiveMutex<T: ?Sized> {
    inner: Mutex<RecursiveMutexInner>,
    value: UnsafeCell<T>,
}

impl<T> RecursiveMutex<T> {
    /// Create a new mutex with value `value`.
    pub fn new(value: T) -> Self {
        RecursiveMutex {
            inner: Mutex::new(RecursiveMutexInner {
                owner: 0,
                count: 0,
            }),
            value: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> RecursiveMutex<T> {
    /// Lock mutex
    pub fn lock(&self) -> RecursiveMutexGuard<T> {
        let pid = {
            let contexts = ::env().contexts.lock();
            if let Some(current) = contexts.current() {
                current.pid
            } else {
                usize::MAX
            }
        };

        loop {
            {
                let mut inner = self.inner.lock();
                if inner.count == 0 || inner.owner == pid {
                    inner.owner = pid;
                    inner.count += 1;
                    break;
                }
            }
            unsafe { context_switch(false) };
        }

        RecursiveMutexGuard::new(&self.inner, &self.value)
    }
}

unsafe impl<T: ?Sized + Send> Send for RecursiveMutex<T> { }

unsafe impl<T: ?Sized + Send> Sync for RecursiveMutex<T> { }

/// A recursive mutex guard (returned by .lock())
pub struct RecursiveMutexGuard<'a, T: ?Sized + 'a> {
    inner: &'a Mutex<RecursiveMutexInner>,
    data: &'a UnsafeCell<T>,
}

impl<'rmutex, T: ?Sized> RecursiveMutexGuard<'rmutex, T> {
    fn new(inner: &'rmutex Mutex<RecursiveMutexInner>, data: &'rmutex UnsafeCell<T>) -> Self {
        RecursiveMutexGuard {
            inner: inner,
            data: data,
        }
    }
}

impl<'rmutex, T: ?Sized> Deref for RecursiveMutexGuard<'rmutex, T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.data.get() }
    }
}

impl<'rmutex, T: ?Sized> DerefMut for RecursiveMutexGuard<'rmutex, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for RecursiveMutexGuard<'a, T> {
    fn drop(&mut self) {
        let mut inner = self.inner.lock();
        inner.count -= 1;
        if inner.count == 0 {
            inner.owner = 0;
        }
    }
}
