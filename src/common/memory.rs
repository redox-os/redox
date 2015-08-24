use core::cmp::min;
use core::mem::size_of;

pub const PAGE_DIRECTORY: usize = 0x300000;
pub const PAGE_TABLE_SIZE: usize = 1024;
pub const PAGE_TABLES: usize = PAGE_DIRECTORY + PAGE_TABLE_SIZE * 4;
pub const PAGE_SIZE: usize = 4*1024;

pub const CLUSTER_ADDRESS: usize = PAGE_TABLES + PAGE_TABLE_SIZE * PAGE_TABLE_SIZE * 4 ;
pub const CLUSTER_COUNT: usize = 1024*1024; // 4 GiB
pub const CLUSTER_SIZE: usize = 4*1024; // Of 4 K chunks

unsafe fn cluster(number: usize) -> usize{
    if number < CLUSTER_COUNT {
        return *((CLUSTER_ADDRESS + number * size_of::<usize>()) as *const usize);
    }else{
        return 0;
    }
}

unsafe fn set_cluster(number: usize, address: usize){
    if number < CLUSTER_COUNT {
        *((CLUSTER_ADDRESS + number * size_of::<usize>()) as *mut usize) = address;
    }
}

unsafe fn address_to_cluster(address: usize) -> usize {
    return (address - CLUSTER_ADDRESS - CLUSTER_COUNT * size_of::<usize>())/CLUSTER_SIZE;
}

unsafe fn cluster_to_address(number: usize) -> usize {
    return CLUSTER_ADDRESS + CLUSTER_COUNT * size_of::<usize>() + number*CLUSTER_SIZE;
}

pub unsafe fn cluster_init(){
    // TODO: Automatic memory detection
    for i in 0..CLUSTER_COUNT {
        set_cluster(i, 0);
    }
}

pub unsafe fn alloc(size: usize) -> usize{
    if size > 0 {
        let mut number = 0;
        let mut count = 0;
        for i in 0..CLUSTER_COUNT {
            if cluster(i) == 0 {
                if count == 0 {
                    number = i;
                }
                count += 1;
                if count*CLUSTER_SIZE > size {
                    break;
                }
            }else{
                count = 0;
            }
        }
        if count*CLUSTER_SIZE > size {
            let address = cluster_to_address(number);
            for i in number..number + count {
                set_cluster(i, address);
            }
            return address;
        }else{
            return 0;
        }
    }else{
        return 0;
    }
}

pub unsafe fn alloc_aligned(size: usize, alignment: usize) -> usize{
    if size > 0 {
        let mut number = 0;
        let mut count = 0;
        for i in 0..CLUSTER_COUNT {
            if cluster(i) == 0 {
                if count == 0 {
                    if cluster_to_address(i) % alignment == 0 {
                        number = i;
                    }else{
                        continue;
                    }
                }
                count += 1;
                if count*CLUSTER_SIZE > size {
                    break;
                }
            }else{
                count = 0;
            }
        }
        if count*CLUSTER_SIZE > size {
            let address = cluster_to_address(number);
            for i in number..number + count {
                set_cluster(i, address);
            }
            return address;
        }else{
            return 0;
        }
    }else{
        return 0;
    }
}

pub unsafe fn alloc_size(ptr: usize) -> usize {
    let mut size = 0;

    if ptr > 0 {
        for i in address_to_cluster(ptr)..CLUSTER_COUNT {
            if cluster(i) == ptr {
                size += CLUSTER_SIZE;
            }
        }
    }

    return size;
}

pub unsafe fn realloc(ptr: usize, size: usize) -> usize {
    if size == 0 {
        if ptr > 0 {
            unalloc(ptr);
        }
        return 0;
    }

    let old_size = alloc_size(ptr);
    if size <= old_size {
        return ptr;
    }else{
        let new = alloc(size);
        if ptr > 0 {
            if new > 0 {
                let copy_size = min(old_size, size);

                let mut i = 0;
                while i < copy_size - size_of::<usize>() {
                    *(new as *mut usize).offset(i as isize) = *(ptr as *const usize).offset(i as isize);
                    i += size_of::<usize>();
                }
                while i < copy_size {
                    *(new as *mut u8).offset(i as isize) = *(ptr as *const u8).offset(i as isize);
                    i += size_of::<u8>();
                }
            }
            unalloc(ptr);
        }
        return new;
    }
}

pub unsafe fn realloc_inplace(ptr: usize, size: usize) -> usize {
    let old_size = alloc_size(ptr);
    if size <= old_size {
        return size;
    }else{
        return old_size;
    }
}

pub unsafe fn unalloc(ptr: usize){
    if ptr > 0 {
        for i in address_to_cluster(ptr)..CLUSTER_COUNT {
            if cluster(i) == ptr {
                set_cluster(i, 0);
            }
        }
    }
}

pub fn memory_used() -> usize{
    let mut ret = 0;
    unsafe{
        for i in 0..CLUSTER_COUNT {
            if cluster(i) != 0 {
                ret += CLUSTER_SIZE;
            }
        }
    }
    return ret;
}

pub fn memory_free() -> usize{
    let mut ret = 0;
    unsafe{
        for i in 0..CLUSTER_COUNT {
            if cluster(i) == 0 {
                ret += CLUSTER_SIZE;
            }
        }
    }
    return ret;
}
