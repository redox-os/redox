use collections::BTreeMap;
use core::cell::UnsafeCell;
use super::WaitCondition;

pub struct WaitMap<K, V> {
    inner: UnsafeCell<BTreeMap<K, V>>,
    condition: WaitCondition
}

impl<K, V> WaitMap<K, V> where K: Ord {
    pub fn new() -> WaitMap<K, V> {
        WaitMap {
            inner: UnsafeCell::new(BTreeMap::new()),
            condition: WaitCondition::new()
        }
    }

    pub unsafe fn inner<'a>(&'a self) -> &'a mut BTreeMap<K, V> {
        &mut *self.inner.get()
    }

    pub fn send(&self, key: K, value: V) {
        unsafe { self.inner() }.insert(key, value);
        self.condition.notify();
    }

    pub fn receive(&self, key: &K) -> V {
        loop {
            if let Some(value) = unsafe { self.inner() }.remove(key) {
                return value;
            }
            self.condition.wait();
        }
    }
}
