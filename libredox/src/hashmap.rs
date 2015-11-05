use super::prelude::v1::*;

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

    /// Push
    pub fn push(&self, elem: T) -> Self {
        LinkedList::Elem(elem, Box::new(LinkedList::Nil))
    }
}

/// A entry in the hash map
pub struct Entry<K, V> {
    data: LinkedList<(K, V)>,
}

impl<K: PartialEq<K>, V> Entry<K, V> {
    pub fn get(&self, key: K) -> Option<&V> {
        let mut cur = self.data.follow();

        loop {
            let next;
            match cur {
                Some(&LinkedList::Elem((ref k, ref v), ref l)) => {
                    if &key == k {
                        return Some(v);
                    } else {
                        next = l.follow();
                    }
                },
                Some(&LinkedList::Nil) | None => {
                    return None;
                },
            }

            cur = next;
        }

    }
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {

        let mut cur = self.data.follow_mut();

        loop {
            cur = match cur {
                Some(x) => match *x {
                    LinkedList::Elem((ref k, ref mut v), ref mut l) => {
                        if &key == k {
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
}

/// A hashtable
pub struct HashMap<K, V> {
    data: [Entry<K, V>; 256],
}
