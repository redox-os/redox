use core::marker::Copy;
use core::mem;
use core::ops::Add;
use core::ops::Drop;
use core::slice;
use core::slice::SliceExt;

use common::memory::*;

pub struct Vector<T> {
    data: *const T,
    length: u32
}

impl <T: Copy> Vector<T> {
    pub fn new() -> Vector<T> {
        Vector::<T> {
            data: 0 as *const T,
            length: 0
        }
    }
    
    pub fn from_slice(s: &[T]) -> Vector<T> {
        let length = s.len() as u32;
        
        if length == 0 {
            return Vector::<T>::new();
        }
        
        let data = alloc(length * (mem::size_of::<T>() as u32));
    
        let mut i = 0;
        for c in s {
            unsafe {
                *((data + i*(mem::size_of::<T>() as u32)) as *mut T) = *c;
            }
            i += 1;
        }
        
        Vector::<T> {
            data: data as *const T,
            length: length
        }
    }
    
    pub fn from_value(value: T) -> Vector<T> {
        let data = alloc(mem::size_of::<T>() as u32);
        
        unsafe {
            *(data as *mut T) = value;
        }
        
        Vector::<T> {
            data: data as *const T,
            length: 1
        }
    }
    
    pub fn sub(&self, start: u32, len: u32) -> Vector<T> {
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
        
        let data = alloc(length * (mem::size_of::<T>() as u32));
    
        for k in i..j {
            unsafe {
                *((data + (k - i)*(mem::size_of::<T>() as u32)) as *mut T) = *(((self.data as u32) + k*(mem::size_of::<T>() as u32)) as *const T);
            }
        }
        
        Vector::<T> {
            data: data as *const T,
            length: length
        }
    }
    
    pub fn len(&self) -> u32 {
        self.length
    }
    
    // TODO: Str trait
    pub fn as_slice(&self) -> &[T] {
        if self.data as u32 == 0 && self.length == 0 {
            &[]
        }else{
            unsafe {
                slice::from_raw_parts(self.data, self.length as usize)
            }
        }
    }
}

impl <T> Drop for Vector<T> {
    fn drop(&mut self){
        unalloc(self.data as u32);
        self.data = 0 as *const T;
        self.length = 0;
    }
}

impl <T: Copy> Add for Vector<T> {
    type Output = Vector<T>;
    fn add(self, other: Vector<T>) -> Vector<T> {
        let length = self.length + other.length;
        
        if length == 0 {
            return Vector::<T>::new();
        }
        
        let data = alloc(length * (mem::size_of::<T>() as u32));
    
        let mut i = 0;
        for c in self.as_slice() {
            unsafe {
                *((data + i*(mem::size_of::<T>() as u32)) as *mut T) = *c;
            }
            i += 1;
        }
        for c in other.as_slice() {
            unsafe {
                *((data + i*(mem::size_of::<T>() as u32)) as *mut T) = *c;
            }
            i += 1;
        }
    
        Vector::<T> {
            data: data as *const T,
            length: length
        }
    }
}

impl <T: Copy> Add<T> for Vector<T> {
    type Output = Vector<T>;
    fn add(self, other: T) -> Vector<T> {
        self + Vector::<T>::from_value(other)
    }
}