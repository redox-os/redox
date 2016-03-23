#[link(name = "c", kind = "static")]
extern {
    fn malloc(size: usize) -> *mut u8;
    fn realloc(ptr: *mut u8, size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}

#[allocator]
#[no_mangle]
pub extern "C" fn __rust_allocate(size: usize, _align: usize) -> *mut u8 {
    unsafe { malloc(size) }
}

#[no_mangle]
pub extern "C" fn __rust_deallocate(ptr: *mut u8, _old_size: usize, _align: usize) {
    unsafe { free(ptr) };
}

#[no_mangle]
pub extern "C" fn __rust_reallocate(ptr: *mut u8, _old_size: usize, size: usize, _align: usize) -> *mut u8 {
    unsafe { realloc(ptr, size) }
}

#[no_mangle]
pub extern "C" fn __rust_reallocate_inplace(_ptr: *mut u8, old_size: usize, _size: usize, _align: usize) -> usize {
    old_size
}

#[no_mangle]
pub extern "C" fn __rust_usable_size(size: usize, _align: usize) -> usize {
    size
}
