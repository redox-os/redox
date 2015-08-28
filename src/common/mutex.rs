use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut, Drop};
use core::sync::atomic::{AtomicBool, Ordering};

use common::debug::*;

use syscall::call::sys_yield;

pub struct Mutex<T: ?Sized> {
    lock: AtomicBool,
    value: UnsafeCell<T>
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Mutex<T> {
        Mutex {
            lock: AtomicBool::new(false),
            value: UnsafeCell::new(value)
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    pub fn lock(&self) -> MutexGuard<T> {
        while self.lock.compare_and_swap(false, true, Ordering::SeqCst) {
            sys_yield();
        }
        return MutexGuard::new(&self.lock, &self.value);
    }
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> { }

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> { }

pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a AtomicBool,
    data: &'a UnsafeCell<T>
}

impl<'mutex, T: ?Sized> MutexGuard<'mutex, T> {
    fn new(lock: &'mutex AtomicBool, data: &'mutex UnsafeCell<T>) -> MutexGuard<'mutex, T> {
        MutexGuard {
            lock: lock,
            data: data
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
        if ! self.lock.compare_and_swap(true, false, Ordering::SeqCst) {
            d("Mutex was already unlocked!\n");
        }
    }
}
