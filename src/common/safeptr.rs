use core::mem::size_of;
use core::ops::Drop;
use core::option::Option;

use common::memory::*;

pub struct SafePtr<T> {
    pub ptr: * mut T
}

impl<T> SafePtr<T> {
    pub fn empty() -> SafePtr<T> {
        SafePtr {
            ptr: 0 as *mut T
        }
    }

    pub fn new() -> SafePtr<T> {
        unsafe {
            SafePtr {
                ptr: alloc(size_of::<T>()) as *mut T
            }
        }
    }

    pub fn get(&self) -> Option<&T> {
        unsafe {
            if self.ptr as usize > 0 {
                return Option::Some(&*self.ptr);
            }else{
                return Option::None;
            }
        }
    }

    pub fn mut_get(&mut self) -> Option<&mut T> {
        unsafe {
            if self.ptr as usize > 0 {
                return Option::Some(&mut *self.ptr);
            }else{
                return Option::None;
            }
        }
    }

    pub unsafe fn unsafe_ptr(&self) -> *const T {
        self.ptr
    }

    pub unsafe fn unsafe_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
}

impl<T> Drop for SafePtr<T> {
    fn drop(&mut self){
        unsafe{
            if self.ptr as usize > 0 {
                unalloc(self.ptr as usize);
                self.ptr = 0 as * mut T;
            }
        }
    }
}