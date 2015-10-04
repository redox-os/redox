use core::clone::Clone;
use core::iter::Iterator;
use core::mem::size_of;
use core::ops::Drop;
use core::option::Option;
use core::ptr;
use core::slice;
use core::slice::SliceExt;

use syscall::call::*;

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
    pub data: *mut T,
    pub length: usize,
}

impl <T> Vec<T> {
    /// Create a empty vector
    pub fn new() -> Vec<T> {
        Vec::<T> {
            data: 0 as *mut T,
            length: 0,
        }
    }

    /// Convert to pointer
    pub unsafe fn as_ptr(&self) -> *const T {
        self.data
    }

    /// Convert from a raw (unsafe) buffer
    pub unsafe fn from_raw_buf(ptr: *const T, len: usize) -> Vec<T> {
        let data = sys_alloc(size_of::<T>() * len) as *mut T;

        ptr::copy(ptr, data, len);

        Vec::<T> {
            data: data,
            length: len,
        }
    }

    pub fn from_slice(slice: &[T]) -> Vec<T> {
        let data;
        unsafe{
            data = sys_alloc(size_of::<T>() * slice.len()) as *mut T;
            
            ptr::copy(slice.as_ptr(), data, slice.len());
        }
        
        Vec::<T> {
            data: data,
            length: slice.len()
        }
    }
    
    
    /// Get the nth element. Returns None if out of bounds.
    pub fn get(&self, i: usize) -> Option<&mut T> {
        if i >= self.length {
            Option::None
        } else {
            unsafe { Option::Some(&mut *self.data.offset(i as isize)) }
        }
    }

    /// Set the nth element
    pub fn set(&self, i: usize, value: T) {
        if i <= self.length {
            unsafe {
                ptr::write(self.data.offset(i as isize), value);
            }
        }
    }

    /// Insert element at a given position
    pub fn insert(&mut self, i: usize, value: T) {
        if i <= self.length {
            self.length += 1;
            unsafe {
                self.data = sys_realloc(self.data as usize, self.length * size_of::<T>()) as *mut T;

                //Move all things ahead of insert forward one
                let mut j = self.length - 1;
                while j > i {
                    ptr::write(self.data.offset(j as isize),
                               ptr::read(self.data.offset(j as isize - 1)));
                    j -= 1;
                }

                ptr::write(self.data.offset(i as isize), value);
            }
        }
    }

    /// Remove a element and return it as a Option
    pub fn remove(&mut self, i: usize) -> Option<T> {
        if i < self.length {
            self.length -= 1;
            unsafe {
                let item = ptr::read(self.data.offset(i as isize));

                //Move all things ahead of remove back one
                let mut j = i;
                while j < self.length {
                    ptr::write(self.data.offset(j as isize),
                               ptr::read(self.data.offset(j as isize + 1)));
                    j += 1;
                }

                self.data = sys_realloc(self.data as usize, self.length * size_of::<T>()) as *mut T;

                Option::Some(item)
            }
        } else {
            Option::None
        }
    }

    /// Push an element to a vector
    pub fn push(&mut self, value: T) {
        self.length += 1;
        unsafe {
            self.data = sys_realloc(self.data as usize, self.length * size_of::<T>()) as *mut T;
            ptr::write(self.data.offset(self.length as isize - 1), value);
        }
    }

    /// Pop the last element
    pub fn pop(&mut self) -> Option<T> {
        if self.length > 0 {
            self.length -= 1;
            unsafe {
                let item = ptr::read(self.data.offset(self.length as isize));
                self.data = sys_realloc(self.data as usize, self.length * size_of::<T>()) as *mut T;

                Option::Some(item)
            }
        } else {
            Option::None
        }
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
    pub fn sub(&self, start: usize, count: usize) -> Vec<T> {
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

        unsafe {
            let data = sys_alloc(length * size_of::<T>()) as *mut T;

            for k in i..j {
                ptr::write(data.offset((k - i) as isize),
                           ptr::read(self.data.offset(k as isize)));
            }

            Vec {
                data: data,
                length: length,
            }
        }
    }

    pub fn as_slice(&self) -> &[T] {
        if self.data as usize > 0 && self.length > 0 {
            unsafe { slice::from_raw_parts(self.data, self.length) }
        } else {
            &[]
        }
    }
}

impl<T> Vec<T> where T: Clone {
    /// Append a vector to another vector
    pub fn push_all(&mut self, vec: &Vec<T>) {
        let mut i = self.length as isize;
        self.length += vec.len();
        unsafe {
            self.data = sys_realloc(self.data as usize, self.length * size_of::<T>()) as *mut T;

            for value in vec.iter() {
                ptr::write(self.data.offset(i), value.clone());
                i += 1;
            }
        }
    }
}

impl<T> Clone for Vec<T> where T: Clone {
    fn clone(&self) -> Vec<T> {
        let mut ret = Vec::new();
        ret.push_all(self);
        ret
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.len() {
                ptr::read(self.data.offset(i as isize));
            }

            sys_unalloc(self.data as usize);
            self.data = 0 as *mut T;
            self.length = 0;
        }
    }
}
