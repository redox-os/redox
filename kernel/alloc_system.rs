use arch::memory::*;

#[allocator]
#[no_mangle]
pub extern "C" fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    if align > 4 {
        debugln!("align: {}", align);
    }
    unsafe { alloc_aligned(size, align) as *mut u8 }
}

#[no_mangle]
pub extern "C" fn __rust_deallocate(ptr: *mut u8, _old_size: usize, _align: usize) {
    unsafe { unalloc(ptr as usize) }
}

#[no_mangle]
pub extern "C" fn __rust_reallocate(ptr: *mut u8, _old_size: usize, size: usize, align: usize) -> *mut u8 {
    unsafe { realloc_aligned(ptr as usize, size, align) as *mut u8 }
}

#[no_mangle]
pub extern "C" fn __rust_reallocate_inplace(ptr: *mut u8, _old_size: usize, size: usize, _align: usize) -> usize {
    unsafe { realloc_inplace(ptr as usize, size) }
}

#[no_mangle]
pub extern "C" fn __rust_usable_size(size: usize, _align: usize) -> usize {
    size
}
