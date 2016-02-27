use arch::memory::*;

#[allocator]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    unsafe { alloc(size) as *mut u8 }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn __rust_deallocate(ptr: *mut u8, old_size: usize, align: usize) {
    unsafe { unalloc(ptr as usize) }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn __rust_reallocate(ptr: *mut u8,
                                    old_size: usize,
                                    size: usize,
                                    align: usize)
                                    -> *mut u8 {
    unsafe { realloc(ptr as usize, size) as *mut u8 }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn __rust_reallocate_inplace(ptr: *mut u8,
                                            old_size: usize,
                                            size: usize,
                                            align: usize)
                                            -> usize {
    unsafe { realloc_inplace(ptr as usize, size) }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn __rust_usable_size(size: usize, align: usize) -> usize {
    size
}
