use collections::BTreeMap;
use core::mem;
use spin::Mutex;

use sync::WaitCondition;

#[derive(Debug)]
pub struct WaitMap<K, V> {
    inner: Mutex<BTreeMap<K, V>>,
    condition: WaitCondition
}

impl<K, V> WaitMap<K, V> where K: Clone + Ord {
    pub fn new() -> WaitMap<K, V> {
        WaitMap {
            inner: Mutex::new(BTreeMap::new()),
            condition: WaitCondition::new()
        }
    }

    pub fn receive_nonblock(&self, key: &K) -> Option<V> {
        self.inner.lock().remove(key)
    }

    pub fn receive(&self, key: &K) -> V {
        loop {
            if let Some(value) = self.receive_nonblock(key) {
                return value;
            }
            self.condition.wait();
        }
    }

    pub fn receive_any_nonblock(&self) -> Option<(K, V)> {
        let mut inner = self.inner.lock();
        if let Some(key) = inner.keys().next().map(|key| key.clone()) {
            inner.remove(&key).map(|value| (key, value))
        } else {
            None
        }
    }

    pub fn receive_any(&self) -> (K, V) {
        loop {
            if let Some(entry) = self.receive_any_nonblock() {
                return entry;
            }
            self.condition.wait();
        }
    }

    pub fn receive_all(&self) -> BTreeMap<K, V> {
        let mut ret = BTreeMap::new();
        mem::swap(&mut ret, &mut *self.inner.lock());
        ret
    }

    pub fn send(&self, key: K, value: V) {
        self.inner.lock().insert(key, value);
        self.condition.notify();
    }
}
