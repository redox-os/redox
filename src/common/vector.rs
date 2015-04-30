use core::marker::Copy;
use core::mem::size_of;
use core::ops::Add;
use core::ops::Drop;
use core::ptr;
use core::result::Result;
use core::slice;
use core::slice::SliceExt;

use common::memory::*;

pub struct Vector<T> {
    data: *mut T,
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
    
    pub fn from_ptr(ptr: *const T) -> Vector<T> {
        let data = alloc(size_of::<T>());
        
        unsafe {
            ptr::copy(ptr, data as *mut T, size_of::<T>());
        }
        
        Vector::<T> {
            data: data as *mut T,
            length: 1
        }
    }
    
    pub fn from_value(value: T) -> Vector<T> {
        let data = alloc(size_of::<T>());
        
        unsafe {
            ptr::write(data as *mut T, value);
        }
        
        Vector::<T> {
            data: data as *mut T,
            length: 1
        }
    }
    
    pub fn get(&self, i: usize) -> Result<&mut T, usize> {
        if i >= self.len() {
            return Result::Err(self.len());
        }else{
            unsafe{
                return Result::Ok(&mut *(((self.data as usize) + i * size_of::<T>()) as *mut T));
            }
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
        
        let data = alloc(length * size_of::<T>());
    
        for k in i..j {
            unsafe {
                ptr::write((data + (k - i) * size_of::<T>()) as *mut T, ptr::read(((self.data as usize) + k* size_of::<T>()) as *const T));
            }
        }
        
        Vector::<T> {
            data: data as *mut T,
            length: length
        }
    }
    
    pub fn len(&self) -> usize {
        self.length
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

impl <T> Drop for Vector<T> {
    fn drop(&mut self){
        unalloc(self.data as usize);
        self.data = 0 as *mut T;
        self.length = 0;
    }
}

impl <T> Add for Vector<T> {
    type Output = Vector<T>;
    fn add(self, other: Vector<T>) -> Vector<T> {
        let length = self.length + other.length;
        
        if length == 0 {
            return Vector::<T>::new();
        }
        
        let data = alloc(length * size_of::<T>());
    
        for i in 0..self.len() {
            unsafe {
                ptr::write((data + i * size_of::<T>()) as *mut T, ptr::read((self.data as usize + i * size_of::<T>()) as *const T));
            }
        }
        
        for i in 0..other.len() {
            unsafe {
                ptr::write((data + (i + self.len()) * size_of::<T>()) as *mut T, ptr::read((other.data as usize + i * size_of::<T>()) as *const T));
            }
        }
    
        Vector::<T> {
            data: data as *mut T,
            length: length
        }
    }
}

impl <T> Add<T> for Vector<T> {
    type Output = Vector<T>;
    fn add(self, other: T) -> Vector<T> {
        self + Vector::<T>::from_value(other)
    }
}