use collections::BTreeMap;
use spin::Mutex;

use sync::WaitCondition;

#[derive(Debug)]
pub struct WaitMap<K, V> {
    inner: Mutex<BTreeMap<K, V>>,
    condition: WaitCondition
}

impl<K, V> WaitMap<K, V> where K: Ord {
    pub fn new() -> WaitMap<K, V> {
        WaitMap {
            inner: Mutex::new(BTreeMap::new()),
            condition: WaitCondition::new()
        }
    }

    pub fn send(&self, key: K, value: V) {
        self.inner.lock().insert(key, value);
        self.condition.notify();
    }

    pub fn receive(&self, key: &K) -> V {
        loop {
            if let Some(value) = self.inner.lock().remove(key) {
                return value;
            }
            self.condition.wait();
        }
    }
}
