use super::prelude::v1::*;

/// An linked list (used for entries)
pub enum LinkedList<'a, T> where T: 'a {
    Elem(T, &'a mut LinkedList<'a, T>),
    Nil,
}

impl<'a, T> LinkedList<'a, T> {
    /// Follow
    pub fn follow(&self) -> Option<&Self> {
        use self::LinkedList::*;
        match *self {
            Elem(_, ref l) => {
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
        match self {
            &mut Elem(_, l) => {
                Some(l)
            },
            &mut Nil => {
                None
            },
        }
    }

    /// Push
    pub fn push(&self, elem: T) -> Self {
        LinkedList::Elem(elem, &LinkedList::Nil)
    }
}

/// A entry in the hash map
pub struct Entry<'a, K, V> where K: 'a, V: 'a {
    data: LinkedList<'a, (K, V)>,
}

impl<'a, K: PartialEq<K>, V> Entry<'a, K, V> {
    pub fn get(&self, key: K) -> Option<&V> {
        let mut cur = self.data.follow();

        loop {
            match cur {
                Some(&LinkedList::Elem((ref k, ref v), ref l)) => {
                    if &key == k {
                        return Some(v);
                    } else {
                        cur = l.follow();
                    }
                },
                Some(&LinkedList::Nil) | None => {
                    return None;
                },
            }
        }

    }
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {

        let mut cur = self.data.follow_mut();

        loop {
            match cur {
                Some(&mut LinkedList::Elem((ref k, ref mut v), ref mut l)) => {
                    if &key == k {
                        return Some(v);
                    } else {
                        cur = l.follow_mut();
                    }
                },
                Some(&mut LinkedList::Nil) | None => {
                    return None;
                },
            }
        }
    }
}

/// A hashtable
pub struct HashMap<'a, K, V> where K: 'a, V: 'a {
    data: [Entry<'a, K, V>; 256],
}
