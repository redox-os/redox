use core::clone::Clone;
use core::mem::size_of;
use core::ops::Add;
use core::ops::Drop;
use core::ptr;
use core::result::Result;
use core::slice;
use core::slice::SliceExt;

use common::memory::*;

pub struct Vector<T> {
    // TODO: Hide!
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
            let data = alloc(size_of::<T>());

            ptr::write(data as *mut T, value);

            Vector::<T> {
                data: data as *mut T,
                length: 1
            }
        }
    }

    pub fn get(&self, i: usize) -> Result<&mut T, usize> {
        if i >= self.len() {
            return Result::Err(self.len());
        }else{
            unsafe{
                return Result::Ok(&mut*self.data.offset(i as isize));
            }
        }
    }

    pub fn len(&self) -> usize {
        self.length
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

    // TODO: Str trait
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
            /* TODO: Destruct values
            for i in 0..self.len() {
                ptr::read(self.data.offset(i as isize));
            }
            */

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

impl <T> Add<T> for Vector<T> {
    type Output = Vector<T>;
    fn add(self, other: T) -> Vector<T> {
        self + Vector::<T>::from_value(other)
    }
}