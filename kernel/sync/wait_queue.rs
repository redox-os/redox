use collections::vec_deque::VecDeque;

use core::cell::UnsafeCell;
use core::mem;
use core::ops::DerefMut;

use common::time::Duration;

use super::WaitCondition;

pub struct WaitQueue<T> {
    pub inner: UnsafeCell<VecDeque<T>>,
    pub condition: WaitCondition,
}

impl<T> WaitQueue<T> {
    pub fn new() -> WaitQueue<T> {
        WaitQueue {
            inner: UnsafeCell::new(VecDeque::new()),
            condition: WaitCondition::new()
        }
    }

    pub unsafe fn inner<'a>(&'a self) -> &'a mut VecDeque<T> {
        &mut *self.inner.get()
    }

    pub fn clone(&self) -> WaitQueue<T> where T: Clone {
        WaitQueue {
            inner: UnsafeCell::new(unsafe { self.inner() }.clone()),
            condition: WaitCondition::new()
        }
    }

    pub fn receive(&self, reason: &str) -> T {
        loop {
            if let Some(value) = unsafe { self.inner() }.pop_front() {
                return value;
            }
            self.condition.wait(reason);
        }
    }

    pub fn receive_for(&self, reason: &str, time: Duration) -> Option<T> {
        loop {
            if let Some(value) = unsafe { self.inner() }.pop_front() {
                return Some(value);
            }
            if ! self.condition.wait_for(reason, time) {
                return None;
            }
        }
    }

    pub fn receive_all(&self, reason: &str) -> VecDeque<T> {
        loop {
            {
                let mut inner = unsafe { self.inner() };
                if ! inner.is_empty() {
                    let mut swap_inner = VecDeque::new();
                    mem::swap(inner.deref_mut(), &mut swap_inner);
                    return swap_inner;
                }
            }
            self.condition.wait(reason);
        }
    }

    pub fn send(&self, value: T, reason: &str) {
        unsafe { self.inner() }.push_back(value);
        self.condition.notify(reason);
    }
}
