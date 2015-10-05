use core::*;
use core::hash::*;
use common::*;

pub struct Entry<T, U> {
    pub keys: Vec<(T, U)>,
    cur: usize,
}

impl<T, U> Iterator<Item = (T, U)> for Entry<T, U> {
    fn next(&mut self) -> Option<(T, U)> {
        self.usize += 1;
        self.keys.get(self.usize)
    }
}

/// A hashmap (a simple implementation)
pub struct HashMap<T, U>
           where T: Hash {
    values: [Entry<T, U>; 247],
}

impl<T, U> HashMap<T, U>
           where T: Hash {
    /// Get the position of an entry
    pub fn get_pos(key: &T) -> u8 {
        let hash = SipHasher::new();

        key.hash(hash);
        hash.finish() % 248
    }

    /// Get a refference to an entry
    pub fn get(&self, key: &T) -> Option<&U> {
        &self.values[self.get_pos(key)]
             .find(|(k, v)| key == k)
    }
    /// Get a mutable refference to an entry
    pub fn get_mut(&mut self, key: &T) -> Option<&mut U> {
        &mut self.values[self.get_pos(key)]
                 .find(|(k, v)| key == k)
    }
    /// Set the value of an entry
    pub fn set(&mut self, key: &T, val: &U) {
        match self.get_mut(key) {
            Some(e) => e,
            None => {
                self.values[self.get_pos(key)]
                    .keys
                    .push((*key, *val));
            },
        }
    }
}
