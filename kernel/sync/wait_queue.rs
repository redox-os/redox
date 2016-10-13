use collections::vec_deque::VecDeque;
use core::mem;
use core::ops::DerefMut;
use spin::Mutex;

use sync::WaitCondition;

#[derive(Debug)]
pub struct WaitQueue<T> {
    pub inner: Mutex<VecDeque<T>>,
    pub condition: WaitCondition,
}

impl<T> WaitQueue<T> {
    pub fn new() -> WaitQueue<T> {
        WaitQueue {
            inner: Mutex::new(VecDeque::new()),
            condition: WaitCondition::new()
        }
    }

    pub fn clone(&self) -> WaitQueue<T> where T: Clone {
        WaitQueue {
            inner: Mutex::new(self.inner.lock().clone()),
            condition: WaitCondition::new()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.lock().is_empty()
    }

    pub fn receive(&self) -> T {
        loop {
            if let Some(value) = self.inner.lock().pop_front() {
                return value;
            }
            self.condition.wait();
        }
    }

    pub fn receive_into(&self, buf: &mut [T]) -> usize {
        let mut i = 0;

        if i < buf.len() {
            buf[i] = self.receive();
            i += 1;
        }

        while i < buf.len() {
            if let Some(value) = self.inner.lock().pop_front() {
                buf[i] = value;
                i += 1;
            } else {
                break;
            }
        }

        i
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
            self.condition.wait();
        }
    }

    pub fn send(&self, value: T) -> usize {
        let len = {
            let mut inner = self.inner.lock();
            inner.push_back(value);
            inner.len()
        };
        self.condition.notify();
        len
    }

    pub fn send_from(&self, buf: &[T]) -> usize where T: Copy {
        let len = {
            let mut inner = self.inner.lock();
            inner.extend(buf.iter());
            inner.len()
        };
        self.condition.notify();
        len
    }
}
