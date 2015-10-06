use core::clone::Clone;
use core::iter::Iterator;
use core::mem::size_of;
use core::ops::Drop;
use core::option::Option;
use core::ptr;
use core::slice;
use core::slice::SliceExt;

use common::memory::*;

#[macro_export]
macro_rules! kvec {
    ($($x:expr),*) => (
        KVec::from_slice(&[$($x),*])
    );
    ($($x:expr,)*) => (kvec![$($x),*])
}

/// An iterator over a kvec
pub struct KVecIterator<'a, T: 'a> {
    kvec: &'a KVec<T>,
    offset: usize,
}

impl <'a, T> Iterator for KVecIterator<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        match self.kvec.get(self.offset) {
            Option::Some(item) => {
                self.offset += 1;
                Option::Some(item)
            }
            Option::None => {
                Option::None
            }
        }
    }
}

/// A owned, heap allocated list of elements
pub struct KVec<T> {
    pub mem: Memory<T>, // TODO: Option<Memory>
    pub length: usize,
}

impl <T> KVec<T> {
    /// Create a empty kvector
    pub fn new() -> Self {
        KVec {
            mem: Memory { ptr: 0 as *mut T /* TODO: Option::None */ },
            length: 0,
        }
    }

    /// Convert to pointer
    pub unsafe fn as_ptr(&self) -> *const T {
        self.mem.ptr
    }

    /// Convert from a raw (unsafe) buffer
    pub unsafe fn from_raw_buf(ptr: *const T, len: usize) -> Self {
        match Memory::new(len) {
            Option::Some(mem) => {
                ptr::copy(ptr, mem.ptr, len);

                return KVec {
                    mem: mem,
                    length: len,
                };
            }
            Option::None => {
                return KVec::new();
            }
        }
    }

    pub fn from_slice(slice: &[T]) -> Self {
        match Memory::new(slice.len()) {
            Option::Some(mem) => {
                unsafe { ptr::copy(slice.as_ptr(), mem.ptr, slice.len()) };

                return KVec {
                    mem: mem,
                    length: slice.len(),
                };
            }
            Option::None => {
                return KVec::new();
            }
        }
    }


    /// Get the nth element. Returns None if out of bounds.
    pub fn get(&self, i: usize) -> Option<&mut T> {
        if i >= self.length {
            Option::None
        } else {
            unsafe { Option::Some(&mut *self.mem.ptr.offset(i as isize)) }
        }
    }

    /// Set the nth element
    pub fn set(&self, i: usize, value: T) {
        if i <= self.length {
            unsafe { ptr::write(self.mem.ptr.offset(i as isize), value) };
        }
    }

    /// Insert element at a given position
    pub fn insert(&mut self, i: usize, value: T) {
        if i <= self.length {
            let new_length = self.length + 1;
            if self.mem.renew(new_length) {
                self.length = new_length;

                //Move all things ahead of insert forward one
                let mut j = self.length - 1;
                while j > i {
                    unsafe {
                        ptr::write(self.mem.ptr.offset(j as isize),
                                   ptr::read(self.mem.ptr.offset(j as isize - 1)));
                    }
                    j -= 1;
                }

                unsafe { ptr::write(self.mem.ptr.offset(i as isize), value) };
            }
        }
    }

    /// Remove a element and return it as a Option
    pub fn remove(&mut self, i: usize) -> Option<T> {
        if i < self.length {
            self.length -= 1;

            let item = unsafe { ptr::read(self.mem.ptr.offset(i as isize)) };

            //Move all things ahead of remove back one
            let mut j = i;
            while j < self.length {
                unsafe {
                    ptr::write(self.mem.ptr.offset(j as isize),
                               ptr::read(self.mem.ptr.offset(j as isize + 1)));
                }
                j += 1;
            }

            self.mem.renew(self.length);

            Option::Some(item)
        } else {
            Option::None
        }
    }

    /// Push an element to a kvector
    pub fn push(&mut self, value: T) {
        let new_length = self.length + 1;
        if self.mem.renew(new_length) {
            self.length = new_length;

            unsafe { ptr::write(self.mem.ptr.offset(self.length as isize - 1), value) };
        }
    }

    /// Pop the last element
    pub fn pop(&mut self) -> Option<T> {
        if self.length > 0 {
            self.length -= 1;

            let item = unsafe { ptr::read(self.mem.ptr.offset(self.length as isize)) };

            self.mem.renew(self.length);

            return Option::Some(item);
        }

        Option::None
    }

    /// Get the length of the kvector
    pub fn len(&self) -> usize {
        self.length
    }

    /// Create an iterator
    pub fn iter(&self) -> KVecIterator<T> {
        KVecIterator {
            kvec: self,
            offset: 0,
        }
    }

    // TODO: Consider returning a slice instead
    pub fn sub(&self, start: usize, count: usize) -> Self {
        let mut i = start;
        if i > self.len() {
            i = self.len();
        }

        let mut j = i + count;
        if j > self.len() {
            j = self.len();
        }

        let length = j - i;
        if length == 0 {
            return KVec::new();
        }

        match Memory::new(length) {
            Option::Some(mem) => {
                for k in i..j {
                    unsafe {
                        ptr::write(mem.ptr.offset((k - i) as isize),
                                   ptr::read(self.mem.ptr.offset(k as isize)))
                    };
                }

                return KVec {
                    mem: mem,
                    length: length,
                };
            }
            Option::None => {
                return KVec::new();
            }
        }
    }

    pub fn as_slice(&self) -> &[T] {
        if self.length > 0 {
            unsafe { slice::from_raw_parts(self.mem.ptr, self.length) }
        } else {
            &[]
        }
    }
}

impl<T> KVec<T> where T: Clone {
    /// Append a kvector to another kvector
    pub fn push_all(&mut self, kvec: &Self) {
        let mut i = self.length as isize;
        let new_length = self.length + kvec.len();
        if self.mem.renew(new_length) {
            self.length = new_length;

            for value in kvec.iter() {
                unsafe { ptr::write(self.mem.ptr.offset(i), value.clone()) };
                i += 1;
            }
        }
    }
}

impl<T> Clone for KVec<T> where T: Clone {
    fn clone(&self) -> Self {
        let mut ret = KVec::new();
        ret.push_all(self);
        ret
    }
}

impl<T> Drop for KVec<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.len() {
                ptr::read(self.mem.ptr.offset(i as isize));
            }
        }
    }
}
