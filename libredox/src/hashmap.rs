use super::prelude::v1::*;
use super::hash::*;
use core::mem;

/// An linked list (used for entries)
pub enum LinkedList<T> {
    Elem(T, Box<LinkedList<T>>),
    Nil,
}

impl<T> LinkedList<T> {
    /// Follow
    pub fn follow(&self) -> Option<&Self> {
        use self::LinkedList::*;
        match *self {
            Elem(_, box ref l) => {
                Some(l)
            },
            Nil => {
                None
            },
        }
    }

    /// Follow mutable
    pub fn follow_mut(&mut self) -> Option<&mut Self> {
        use self::LinkedList::*;
        match *self {
            Elem(_, box ref mut l) => {
                Some(l)
            },
            Nil => {
                None
            },
        }
    }

    /// Push (consumes the list)
    pub fn push(self, elem: T) -> Self {
        LinkedList::Elem(elem, Box::new(self))
    }
}

/// A entry in the hash map
pub struct Entry<K, V> {
    data: LinkedList<(K, V)>,
}

impl<K: PartialEq<K>, V> Entry<K, V> {
    /// Create new
    pub fn new(key: K, value: V) -> Self {
        Entry {
            data: LinkedList::Elem((key, value), Box::new(LinkedList::Nil)),
        }
    }

    /// Get value from entry
    pub fn get(&self, key: &K) -> Option<&V> {
        let mut cur = self.data.follow();

        loop {
            cur = match cur {
                Some(&LinkedList::Elem((ref k, ref v), ref l)) => {
                    if key == k {
                        return Some(v);
                    } else {
                        l.follow()
                    }
                },
                Some(&LinkedList::Nil) | None => {
                    return None;
                },
            }
        }

    }

    /// Get value mutable from entry
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {

        let mut cur = self.data.follow_mut();

        loop {
            cur = match cur {
                Some(x) => match *x {
                    LinkedList::Elem((ref k, ref mut v), ref mut l) => {
                        if key == k {
                            return Some(v);
                        } else {
                            l.follow_mut()
                        }
                    },
                    LinkedList::Nil => {
                        return None;
                    },
                },
                None => {
                    return None;
                },
            }

        }
    }

    /// Push to the list (consumes the entry returning the new one)
    pub fn push(self, key: K, value: V) -> Self {
        Entry {
            data: self.data.push((key, value)),
        }
    }
}

/// A hashtable
pub struct HashMap<K, V> {
    data: [Entry<K, V>; 256],
}

impl<K: Hash + PartialEq, V> HashMap<K, V> {
    /// Get value
    pub fn get(&self, key: &K) -> Option<&V> {
        let mut s = Djb2::new();
        key.hash(&mut s);
        self.data[(s.finish() % 256) as usize].get(key)
    }
    /// Get value mutable
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let mut s = Djb2::new();
        key.hash(&mut s);
        self.data[(s.finish() % 256) as usize].get_mut(key)
    }
    /// Set value
    pub fn set(&mut self, key: K, val: V) {
        let mut s = Djb2::new();
        key.hash(&mut s);
        match self.get_mut(&key) {
            Some(k) => {
                *k = val;
                return;
            },
            _ => {}
        }
        let n = (s.finish() % 256) as usize;
        mem::replace(&mut self.data[n], self.data[n].push(key, val));
    }
}

/// DJB2 hashing
pub struct Djb2 {
    state: u64,
}

impl Djb2 {
    /// Create new DJB2 hasher
    pub fn new() -> Self {
        Djb2 {
            state: 5381,
        }
    }
}

impl Hasher for Djb2 {
    fn finish(&self) -> u64 {
        self.state
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.state = ((self.state << 5) + self.state) + b as u64;
        }
    }
}
