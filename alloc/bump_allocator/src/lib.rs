//! A simple allocator that never frees, for testing

#![feature(allocator)]
#![allocator]
#![no_std]

pub static mut HEAP: usize = 10*1024*1024;

#[no_mangle]
pub extern fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    unsafe {
        let mut ret = HEAP;
        if align.is_power_of_two() {
            ret += (align - 1);
            ret &= !(align - 1);
        } else {
            assert_eq!(align, 0);
        }
        HEAP = ret + size;
        ret as *mut u8
    }
}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, align: usize) -> usize {
    size
}

#[no_mangle]
pub extern fn __rust_deallocate(ptr: *mut u8, size: usize, align: usize) {

}

#[no_mangle]
pub extern fn __rust_reallocate(ptr: *mut u8, size: usize, new_size: usize, align: usize) -> *mut u8 {
    use core::{ptr, cmp};

    let new_ptr = __rust_allocate(new_size, align);
    unsafe { ptr::copy(ptr, new_ptr, cmp::min(size, new_size)) };
    __rust_deallocate(ptr, size, align);
    new_ptr
}

#[no_mangle]
pub extern fn __rust_reallocate_inplace(ptr: *mut u8, size: usize, new_size: usize, align: usize) -> usize {
    size
}
