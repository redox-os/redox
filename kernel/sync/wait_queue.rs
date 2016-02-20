use collections::vec_deque::VecDeque;

use core::mem;
use core::ops::DerefMut;

use super::Intex;
use super::WaitCondition;

pub struct WaitQueue<T> {
    pub inner: Intex<VecDeque<T>>,
    pub condition: WaitCondition,
}

impl<T> WaitQueue<T> {
    pub fn new() -> WaitQueue<T> {
        WaitQueue {
            inner: Intex::new(VecDeque::new()),
            condition: WaitCondition::new()
        }
    }

    pub fn receive(&self) -> T {
        loop {
            if let Some(value) = self.inner.lock().pop_front() {
                return value;
            }
            unsafe { self.condition.wait(); }
        }
    }

    pub fn receive_all(&self) -> VecDeque<T> {
        loop {
            {
                let mut inner = self.inner.lock();
                if ! inner.is_empty() {
                    let mut swap_inner = VecDeque::new();
                    mem::swap(inner.deref_mut(), &mut swap_inner);
                    return swap_inner;
                }
            }
            unsafe { self.condition.wait(); }
        }
    }

    pub fn send(&self, value: T) {
        self.inner.lock().push_back(value);
        unsafe { self.condition.notify(); }
    }
}
