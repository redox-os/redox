use core::mem;
use common::memory::*;
use core::ptr;
use core::intrinsics;
use core::ops::{IndexMut, Index};

/// A wrapper around raw pointers
pub struct Memory<T> {
    pub ptr: *mut T,
}

impl<T> Memory<T> {
    /// Create an empty
    pub fn new(count: usize) -> Option<Self> {
        let alloc = unsafe { alloc(count * mem::size_of::<T>()) };
        if alloc > 0 {
            Some(Memory { ptr: alloc as *mut T })
        } else {
            None
        }
    }

    pub fn new_align(count: usize, align: usize) -> Option<Self> {
        let alloc = unsafe { alloc_aligned(count * mem::size_of::<T>(), align) };
        if alloc > 0 {
            Some(Memory { ptr: alloc as *mut T })
        } else {
            None
        }
    }

    /// Renew the memory
    pub fn renew(&mut self, count: usize) -> bool {
        let address = unsafe { realloc(self.ptr as usize, count * mem::size_of::<T>()) };
        if address > 0 {
            self.ptr = address as *mut T;
            true
        } else {
            false
        }
    }

    /// Get the size in bytes
    pub fn size(&self) -> usize {
        unsafe { alloc_size(self.ptr as usize) }
    }

    /// Get the length in T elements
    pub fn length(&self) -> usize {
        unsafe { alloc_size(self.ptr as usize) / mem::size_of::<T>() }
    }

    /// Get the address
    pub unsafe fn address(&self) -> usize {
        self.ptr as usize
    }

    /// Read the memory
    pub unsafe fn read(&self, i: usize) -> T {
        ptr::read(self.ptr.offset(i as isize))
    }

    /// Load the memory
    pub unsafe fn load(&self, i: usize) -> T {
        intrinsics::atomic_singlethreadfence();
        ptr::read(self.ptr.offset(i as isize))
    }

    /// Overwrite the memory
    pub unsafe fn write(&mut self, i: usize, value: T) {
        ptr::write(self.ptr.offset(i as isize), value);
    }

    /// Store the memory
    pub unsafe fn store(&mut self, i: usize, value: T) {
        intrinsics::atomic_singlethreadfence();
        ptr::write(self.ptr.offset(i as isize), value)
    }

    /// Convert into a raw pointer
    pub unsafe fn into_raw(&mut self) -> *mut T {
        let ptr = self.ptr;
        self.ptr = 0 as *mut T;
        ptr
    }
}

impl<T> Drop for Memory<T> {
    fn drop(&mut self) {
        if self.ptr as usize > 0 {
            unsafe { dealloc(self.ptr as usize) };
        }
    }
}

impl<T> Index<usize> for Memory<T> {
    type Output = T;

    fn index<'a>(&'a self, _index: usize) -> &'a T {
        unsafe { &*self.ptr.offset(_index as isize) }
    }
}

impl<T> IndexMut<usize> for Memory<T> {
    fn index_mut<'a>(&'a mut self, _index: usize) -> &'a mut T {
        unsafe { &mut *self.ptr.offset(_index as isize) }
    }
}
