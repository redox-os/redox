use core::clone::Clone;
use core::iter::Iterator;
use core::ops::Drop;
use core::option::Option;
use core::ptr;
use core::slice::{self, SliceExt};

use common::memory::*;

#[macro_export]
macro_rules! vec {
    ($($x:expr),*) => (
        Vec::from_slice(&[$($x),*])
    );
    ($($x:expr,)*) => (vec![$($x),*])
}

/// An iterator over a vec
pub struct VecIterator<'a, T: 'a> {
    vec: &'a Vec<T>,
    offset: usize,
}

impl <'a, T> Iterator for VecIterator<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        match self.vec.get(self.offset) {
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
pub struct Vec<T> {
    pub mem: Memory<T>, // TODO: Option<Memory>
    pub length: usize,
}

impl <T> Vec<T> {
    /// Create a empty vector
    pub fn new() -> Self {
        Vec {
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

                return Vec {
                    mem: mem,
                    length: len,
                };
            }
            Option::None => {
                return Self::new();
            }
        }
    }

    pub fn from_slice(slice: &[T]) -> Self {
        match Memory::new(slice.len()) {
            Option::Some(mem) => {
                unsafe { ptr::copy(slice.as_ptr(), mem.ptr, slice.len()) };

                return Vec {
                    mem: mem,
                    length: slice.len(),
                };
            }
            Option::None => {
                return Vec::new();
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

    /// Push an element to a vector
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

    /// Get the length of the vector
    pub fn len(&self) -> usize {
        self.length
    }

    /// Create an iterator
    pub fn iter(&self) -> VecIterator<T> {
        VecIterator {
            vec: self,
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
            return Vec::new();
        }

        match Memory::new(length) {
            Option::Some(mem) => {
                for k in i..j {
                    unsafe {
                        ptr::write(mem.ptr.offset((k - i) as isize),
                                   ptr::read(self.mem.ptr.offset(k as isize)))
                    };
                }

                return Vec {
                    mem: mem,
                    length: length,
                };
            }
            Option::None => {
                return Self::new();
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

impl<T> Vec<T> where T: Clone {
    /// Append a vector to another vector
    pub fn push_all(&mut self, vec: &Self) {
        let mut i = self.length as isize;
        let new_length = self.length + vec.len();
        if self.mem.renew(new_length) {
            self.length = new_length;

            for value in vec.iter() {
                unsafe { ptr::write(self.mem.ptr.offset(i), value.clone()) };
                i += 1;
            }
        }
    }
}

impl<T> Clone for Vec<T> where T: Clone {
    fn clone(&self) -> Self {
        let mut ret = Self::new();
        ret.push_all(self);
        ret
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.len() {
                ptr::read(self.mem.ptr.offset(i as isize));
            }
        }
    }
}
