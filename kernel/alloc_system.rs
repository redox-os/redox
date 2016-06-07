use arch::memory::*;
use arch::paging::Page;

#[allocator]
#[no_mangle]
pub extern fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    unsafe {
        let address = alloc_aligned(size, align);
        if address > 0 {
            for page in 0..(size + CLUSTER_SIZE - 1) / CLUSTER_SIZE {
                let physical_address = address + page * CLUSTER_SIZE;
                let virtual_address = physical_address + LOGICAL_OFFSET;
                Page::new(virtual_address).map_kernel_write(physical_address);
            }

            (address + LOGICAL_OFFSET) as *mut u8
        } else {
            address as *mut u8
        }
    }
}

#[no_mangle]
pub extern fn __rust_deallocate(ptr: *mut u8, old_size: usize, _align: usize) {
    unsafe {
        let address = ptr as usize - LOGICAL_OFFSET;

        unalloc(address);

        for page in 0..(old_size + CLUSTER_SIZE - 1) / CLUSTER_SIZE {
            let physical_address = address + page * CLUSTER_SIZE;
            let virtual_address = physical_address + LOGICAL_OFFSET;
            Page::new(virtual_address).unmap();
        }
    }
}

#[no_mangle]
pub extern fn __rust_reallocate(ptr: *mut u8,
                                old_size: usize,
                                size: usize,
                                align: usize)
                                -> *mut u8 {
    unsafe {
        let old_address = ptr as usize - LOGICAL_OFFSET;
        let address = realloc_aligned(old_address, size, align);

        if address > 0 {
            if address != old_address {
                for page in 0..(old_size + CLUSTER_SIZE - 1) / CLUSTER_SIZE {
                    let physical_address = old_address + page * CLUSTER_SIZE;
                    let virtual_address = physical_address + LOGICAL_OFFSET;
                    Page::new(virtual_address).unmap();
                }

                for page in 0..(size + CLUSTER_SIZE - 1) / CLUSTER_SIZE {
                    let physical_address = address + page * CLUSTER_SIZE;
                    let virtual_address = physical_address + LOGICAL_OFFSET;
                    Page::new(virtual_address).map_kernel_write(physical_address);
                }
            }

            (address + LOGICAL_OFFSET) as *mut u8
        } else {
            address as *mut u8
        }
    }
}

#[no_mangle]
pub extern fn __rust_reallocate_inplace(ptr: *mut u8,
                                        _old_size: usize,
                                        size: usize,
                                        _align: usize)
                                        -> usize {
    unsafe { realloc_inplace(ptr as usize, size) }
}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, _align: usize) -> usize {
    size
}
