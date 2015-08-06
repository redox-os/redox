use core::clone::Clone;
use core::iter::Iterator;
use core::mem::size_of;
use core::ops::Add;
use core::ops::Drop;
use core::option::Option;
use core::ptr;
use core::result::Result;
use core::slice;
use core::slice::SliceExt;

use common::memory::*;

struct VectorIterator<'a, T: 'a> {
    vector: &'a Vector<T>,
    offset: usize
}

impl <'a, T> Iterator for VectorIterator<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item>{
        match self.vector.get(self.offset) {
            Result::Ok(item) => {
                self.offset += 1;
                return Option::Some(item);
            },
            Result::Err(_) => {
                return Option::None;
            }
        }
    }
}

pub struct Vector<T> {
    pub data: *mut T,
    length: usize
}

impl <T> Vector<T> {
    pub fn new() -> Vector<T> {
        Vector::<T> {
            data: 0 as *mut T,
            length: 0
        }
    }

    /*
    pub fn from_slice(s: &[T]) -> Vector<T> {
        let length = s.len();

        if length == 0 {
            return Vector::<T>::new();
        }

        let data = alloc(length * (size_of::<T>()));

        let mut i = 0;
        for c in s {
            unsafe {
                *((data + i*(size_of::<T>())) as *mut T) = *c;
            }
            i += 1;
        }

        Vector::<T> {
            data: data as *const T,
            length: length
        }
    }
    */

    pub unsafe fn from_raw(ptr: *const T, len: usize) -> Vector<T> {
        let data = alloc(size_of::<T>() * len);

        ptr::copy(ptr, data as *mut T, size_of::<T>() * len);

        Vector::<T> {
            data: data as *mut T,
            length: len
        }
    }

    pub unsafe fn from_ptr(ptr: *const T) -> Vector<T> {
        let data = alloc(size_of::<T>());

        ptr::copy(ptr, data as *mut T, size_of::<T>());

        Vector::<T> {
            data: data as *mut T,
            length: 1
        }
    }

    pub fn from_value(value: T) -> Vector<T> {
        unsafe {
            let data = alloc(size_of::<T>()) as *mut T;

            ptr::write(data, value);

            Vector::<T> {
                data: data,
                length: 1
            }
        }
    }

    pub fn get(&self, i: usize) -> Result<&mut T, usize> {
        if i >= self.length {
            return Result::Err(self.length);
        }else{
            unsafe{
                return Result::Ok(&mut*self.data.offset(i as isize));
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

    pub fn extract(&mut self, i: usize) -> Result<T, usize> {
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

                return Result::Ok(item);
            }
        }else{
            return Result::Err(self.length);
        }
    }

    pub fn erase(&mut self, i: usize){
        if i < self.length {
            self.length -= 1;
            unsafe{
                ptr::read(self.data.offset(i as isize));

                //Move all things ahead of remove back one
                let mut j = i;
                while j < self.length {
                    ptr::write(self.data.offset(j as isize), ptr::read(self.data.offset(j as isize + 1)));
                    j += 1;
                }

                self.data = realloc(self.data as usize, self.length * size_of::<T>()) as *mut T;
            }
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

    pub fn iter(&self) -> VectorIterator<T> {
        VectorIterator {
            vector: self,
            offset: 0
        }
    }

    pub fn sub(&self, start: usize, len: usize) -> Vector<T> {
        let mut i = start;
        if i > self.len() {
            i = self.len();
        }

        let mut j = i + len;
        if j > self.len() {
            j = self.len();
        }

        let length = j - i;
        if length == 0 {
            return Vector::<T>::new();
        }

        unsafe {
            let data = alloc(length * size_of::<T>()) as *mut T;

            for k in i..j {
                ptr::write(data.offset((k - i) as isize), ptr::read(self.data.offset(k as isize)));
            }

            Vector {
                data: data,
                length: length
            }
        }
    }

    //TODO: Deprecate
    pub fn as_slice(&self) -> &mut [T] {
        if self.data as usize == 0 && self.length == 0 {
            &mut []
        }else{
            unsafe {
                slice::from_raw_parts_mut(self.data, self.length)
            }
        }
    }
}

impl <T> Clone for Vector<T> {
    fn clone(&self) -> Vector<T> {
        self.sub(0, self.len())
    }
}

impl <T> Drop for Vector<T> {
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

impl <T> Add for Vector<T> {
    type Output = Vector<T>;
    fn add(self, other: Vector<T>) -> Vector<T> {
        let length = self.length + other.length;

        if length == 0 {
            return Vector::<T>::new();
        }

        unsafe{
            let data = alloc(length * size_of::<T>()) as *mut T;

            for i in 0..self.len() {
                ptr::write(data.offset(i as isize), ptr::read(self.data.offset(i as isize)));
            }

            for i in 0..other.len() {
                ptr::write(data.offset((i + self.len()) as isize), ptr::read(other.data.offset(i as isize)));
            }

            Vector::<T> {
                data: data,
                length: length
            }
        }
    }
}
