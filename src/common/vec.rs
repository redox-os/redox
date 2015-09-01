use core::clone::Clone;
use core::iter::Iterator;
use core::mem::size_of;
use core::ops::Drop;
use core::option::Option;
use core::ptr;
use core::slice;
use core::slice::SliceExt;

use common::memory::*;

struct VecIterator<'a, T: 'a> {
    vec: &'a Vec<T>,
    offset: usize
}

impl <'a, T> Iterator for VecIterator<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item>{
        match self.vec.get(self.offset) {
            Option::Some(item) => {
                self.offset += 1;
                return Option::Some(item);
            },
            Option::None => {
                return Option::None;
            }
        }
    }
}

pub struct Vec<T> {
    pub data: *mut T,
    pub length: usize
}

impl <T> Vec<T> {
    pub fn new() -> Vec<T> {
        Vec::<T> {
            data: 0 as *mut T,
            length: 0
        }
    }

    pub unsafe fn as_ptr(&self) -> *const T {
        return self.data;
    }

    pub unsafe fn from_raw_buf(ptr: *const T, len: usize) -> Vec<T> {
        let data = alloc(size_of::<T>() * len);

        ptr::copy(ptr, data as *mut T, len);

        Vec::<T> {
            data: data as *mut T,
            length: len
        }
    }

    pub fn get(&self, i: usize) -> Option<&mut T> {
        if i >= self.length {
            return Option::None;
        }else{
            unsafe{
                return Option::Some(&mut *self.data.offset(i as isize));
            }
        }
    }

    pub fn set(&self, i: usize, value: T) {
        if i <= self.length {
            unsafe {
                ptr::write(self.data.offset(i as isize), value);
            }
        }
    }

    pub fn insert(&mut self, i: usize, value: T) {
        if i <= self.length {
            self.length += 1;
            unsafe {
                self.data = realloc(self.data as usize, self.length * size_of::<T>()) as *mut T;

                //Move all things ahead of insert forward one
                let mut j = self.length - 1;
                while j > i {
                    ptr::write(self.data.offset(j as isize), ptr::read(self.data.offset(j as isize - 1)));
                    j -= 1;
                }

                ptr::write(self.data.offset(i as isize), value);
            }
        }
    }

    pub fn remove(&mut self, i: usize) -> Option<T> {
        if i < self.length {
            self.length -= 1;
            unsafe{
                let item = ptr::read(self.data.offset(i as isize));

                //Move all things ahead of remove back one
                let mut j = i;
                while j < self.length {
                    ptr::write(self.data.offset(j as isize), ptr::read(self.data.offset(j as isize + 1)));
                    j += 1;
                }

                self.data = realloc(self.data as usize, self.length * size_of::<T>()) as *mut T;

                return Option::Some(item);
            }
        }else{
            return Option::None;
        }
    }

    pub fn push(&mut self, value: T) {
        self.length += 1;
        unsafe{
            self.data = realloc(self.data as usize, self.length * size_of::<T>()) as *mut T;
            ptr::write(self.data.offset(self.length as isize - 1), value);
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn iter(&self) -> VecIterator<T> {
        VecIterator {
            vec: self,
            offset: 0
        }
    }

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
            let data = alloc(length * size_of::<T>()) as *mut T;

            for k in i..j {
                ptr::write(data.offset((k - i) as isize), ptr::read(self.data.offset(k as isize)));
            }

            Vec {
                data: data,
                length: length
            }
        }
    }

    pub fn as_slice(&self) -> &[T] {
        if self.data as usize > 0 && self.length > 0 {
            unsafe{
                return slice::from_raw_parts(self.data, self.length);
            }
        }else{
            return &[]
        }
    }
}

impl<T> Vec<T> where T: Clone {
    pub fn push_all(&mut self, vec: &Vec<T>) {
        let mut i = self.length as isize;
        self.length += vec.len();
        unsafe{
            self.data = realloc(self.data as usize, self.length * size_of::<T>()) as *mut T;

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
        return ret;
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self){
        unsafe {
            for i in 0..self.len() {
                ptr::read(self.data.offset(i as isize));
            }

            unalloc(self.data as usize);
            self.data = 0 as *mut T;
            self.length = 0;
        }
    }
}
