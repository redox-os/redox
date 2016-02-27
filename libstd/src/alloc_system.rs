use system::syscall::{sys_alloc, sys_unalloc, sys_realloc, sys_realloc_inplace};

#[allocator]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    unsafe { sys_alloc(size).unwrap() as *mut u8 }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn __rust_deallocate(ptr: *mut u8, old_size: usize, align: usize) {
    unsafe { sys_unalloc(ptr as usize).unwrap() };
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn __rust_reallocate(ptr: *mut u8,
                                    old_size: usize,
                                    size: usize,
                                    align: usize)
                                    -> *mut u8 {
    unsafe { sys_realloc(ptr as usize, size).unwrap() as *mut u8 }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn __rust_reallocate_inplace(ptr: *mut u8,
                                            old_size: usize,
                                            size: usize,
                                            align: usize)
                                            -> usize {
    unsafe { sys_realloc_inplace(ptr as usize, size).unwrap() }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn __rust_usable_size(size: usize, align: usize) -> usize {
    size
}
