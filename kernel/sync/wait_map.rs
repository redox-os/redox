use collections::BTreeMap;

use super::Intex;
use super::WaitCondition;

pub struct WaitMap<K, V> {
    pub inner: Intex<BTreeMap<K, V>>,
    pub condition: WaitCondition
}

impl<K, V> WaitMap<K, V> where K: Ord {
    pub fn new() -> WaitMap<K, V> {
        WaitMap {
            inner: Intex::new(BTreeMap::new()),
            condition: WaitCondition::new()
        }
    }

    pub fn send(&self, key: K, value: V) {
        self.inner.lock().insert(key, value);
        unsafe { self.condition.notify(); }
    }

    pub fn receive(&self, key: &K) -> V {
        loop {
            if let Some(value) = self.inner.lock().remove(key) {
                return value;
            }
            unsafe { self.condition.wait(); }
        }
    }
}
