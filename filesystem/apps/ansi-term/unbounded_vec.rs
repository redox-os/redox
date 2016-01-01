use std::ops::{Index, IndexMut, Deref, DerefMut, RangeArguments};

pub struct UnboundedVec<T> {
    // Evil hack
    none: T,
    vec: Vec<T>,
}

impl<T: Default> UnboundedVec<T> {
    pub fn new() -> Self {
        UnboundedVec {
            vec: Vec::new(),
            none: T::default(),
        }
    }
}

impl UnboundedVec<T> {
    fn extend(&mut self, index: usize) {
        for i in 0..index - self.vec.len() {
            self.vec.push(self.none);
        }
    }
}

impl<T: PartialEq<T>> Deref for UnboundedVec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.vec[self.len() - 1..]
    }
}

impl<T: PartialEq<T>> DerefMut for UnboundedVec<T> {
    type Target = [T];

    fn deref(&self) -> &mut [T] {
        &mut self.vec[self.len() - 1..]
    }
}

impl<T: PartialEq<T>> UnboundedVec<T> {
    pub fn len(&self) -> usize {
        self.vec.iter().rev().skip_while(|x| x == self.none).count()
    }

    pub fn push(&mut self, item: T) {
        let len = self.len();
        if self.vec.len() < len {
            self.vec[len] = item;
        } else {
            self.vec.push(item);
        }
    }

    pub fn take<R: RangeArgument<usize>>(&mut self, range: R) {
        for _ in self.vec.drain(range) {}
    }
}

impl Index<usize> for UnboundedVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.vec.get(index).unwrap_or(&self.none)
    }
}

impl IndexMut<usize> for UnboundedVec<T> {
    type Output = T;

    fn index(&mut self, index: usize) -> &mut T {
        match self.get_mut(index) {
            Some(x) => x,
            None => {
                self.extend();

                &mut self.vec[index]
            }
        }
    }
}
