use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut, Drop};
use core::sync::atomic::{AtomicBool, Ordering};

use thread;

/// A mutex, i.e. a form of safe shared memory between threads. See rust std's Mutex.
pub struct Mutex<T: ?Sized> {
    lock: AtomicBool,
    value: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    /// Create a new mutex with value `value`.
    pub fn new(value: T) -> Self {
        Mutex {
            lock: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Lock the mutex
    pub fn lock(&self) -> Result<MutexGuard<T>, ()> {
        while self.lock.compare_and_swap(false, true, Ordering::SeqCst) {
            thread::yield_now();
        }
        Ok(MutexGuard::new(&self.lock, &self.value))
    }
}

struct Dummy(UnsafeCell<()>);
unsafe impl Sync for Dummy {}
static DUMMY: Dummy = Dummy(UnsafeCell::new(()));

pub struct StaticMutex {
    lock: AtomicBool,
}

impl StaticMutex {
    /// Create a new mutex with value `value`.
    pub const fn new() -> Self {
        StaticMutex {
            lock: AtomicBool::new(false),
        }
    }

    /// Lock the mutex
    pub fn lock(&'static self) -> Result<MutexGuard<()>, ()> {
        while self.lock.compare_and_swap(false, true, Ordering::SeqCst) {
            thread::yield_now();
        }
        Ok(MutexGuard::new(&self.lock, &DUMMY.0)) // TODO catch panics
    }

    pub unsafe fn destroy(&'static self) {
        if !self.lock.compare_and_swap(true, false, Ordering::SeqCst) {
            // Mutex was already unlocked!
        }
    }
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

/// A mutex guard (returned by .lock())
pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a AtomicBool,
    data: &'a UnsafeCell<T>,
}

impl<'mutex, T: ?Sized> MutexGuard<'mutex, T> {
    fn new(lock: &'mutex AtomicBool, data: &'mutex UnsafeCell<T>) -> Self {
        MutexGuard {
            lock: lock,
            data: data,
        }
    }
}

impl<'mutex, T: ?Sized> Deref for MutexGuard<'mutex, T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.data.get() }
    }
}

impl<'mutex, T: ?Sized> DerefMut for MutexGuard<'mutex, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        if !self.lock.compare_and_swap(true, false, Ordering::SeqCst) {
            // Mutex was already unlocked!
        }
    }
}
