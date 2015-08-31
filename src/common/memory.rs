use core::cmp::min;
use core::mem::size_of;
use core::ptr;

use common::scheduler::*;

pub const PAGE_DIRECTORY: usize = 0x300000;
pub const PAGE_TABLE_SIZE: usize = 1024;
pub const PAGE_TABLES: usize = PAGE_DIRECTORY + PAGE_TABLE_SIZE * 4;
pub const PAGE_SIZE: usize = 4*1024;

pub const CLUSTER_ADDRESS: usize = PAGE_TABLES + PAGE_TABLE_SIZE * PAGE_TABLE_SIZE * 4 ;
pub const CLUSTER_COUNT: usize = 1024*1024; // 4 GiB
pub const CLUSTER_SIZE: usize = 4*1024; // Of 4 K chunks

struct MemoryMapEntry {
    base: u64,
    len: u64,
    class: u32,
    acpi: u32
}

const MEMORY_MAP: *const MemoryMapEntry = 0x500 as *const MemoryMapEntry;

unsafe fn cluster(number: usize) -> usize{
    if number < CLUSTER_COUNT {
        return ptr::read((CLUSTER_ADDRESS + number * size_of::<usize>()) as *const usize);
    }else{
        return 0;
    }
}

unsafe fn set_cluster(number: usize, address: usize){
    if number < CLUSTER_COUNT {
        ptr::write((CLUSTER_ADDRESS + number * size_of::<usize>()) as *mut usize, address);
    }
}

unsafe fn address_to_cluster(address: usize) -> usize {
    return (address - CLUSTER_ADDRESS - CLUSTER_COUNT * size_of::<usize>())/CLUSTER_SIZE;
}

unsafe fn cluster_to_address(number: usize) -> usize {
    return CLUSTER_ADDRESS + CLUSTER_COUNT * size_of::<usize>() + number*CLUSTER_SIZE;
}

pub unsafe fn cluster_init(){
    //First, set all clusters to the not present value
    for cluster in 0..CLUSTER_COUNT {
        set_cluster(cluster, 0xFFFFFFFF);
    }

    //Next, set all valid clusters to the free value
    //TODO: Optimize this function
    for i in 0..((0x5000 - 0x500)/size_of::<MemoryMapEntry>()) {
        let entry = &*MEMORY_MAP.offset(i as isize);
        if entry.len > 0 && entry.class == 1 {
            for cluster in 0..CLUSTER_COUNT {
                let address = cluster_to_address(cluster);
                if address as u64 >= entry.base && (address as u64 + CLUSTER_SIZE as u64) <= (entry.base + entry.len) {
                    set_cluster(cluster, 0);
                }
            }
        }
    }
}

pub unsafe fn alloc(size: usize) -> usize {
    let mut ret = 0;

    //Memory allocation must be atomic
    let reenable = start_no_ints();

    if size > 0 {
        let mut number = 0;
        let mut count = 0;

        for i in 0..CLUSTER_COUNT {
            if cluster(i) == 0 {
                if count == 0 {
                    number = i;
                }
                count += 1;
                if count * CLUSTER_SIZE > size {
                    break;
                }
            }else{
                count = 0;
            }
        }
        if count * CLUSTER_SIZE > size {
            let address = cluster_to_address(number);
            for i in number..number + count {
                set_cluster(i, address);
            }
            ret = address;
        }
    }

    //Memory allocation must be atomic
    end_no_ints(reenable);

    return ret;
}

pub unsafe fn alloc_type<T>() -> *mut T {
    return alloc(size_of::<T>()) as *mut T;
}

pub unsafe fn alloc_size(ptr: usize) -> usize {
    let mut size = 0;

    //Memory allocation must be atomic
    let reenable = start_no_ints();

    if ptr > 0 {
        for i in address_to_cluster(ptr)..CLUSTER_COUNT {
            if cluster(i) == ptr {
                size += CLUSTER_SIZE;
            }
        }
    }

    //Memory allocation must be atomic
    end_no_ints(reenable);

    return size;
}

pub unsafe fn unalloc(ptr: usize){
    //Memory allocation must be atomic
    let reenable = start_no_ints();

    if ptr > 0 {
        for i in address_to_cluster(ptr)..CLUSTER_COUNT {
            if cluster(i) == ptr {
                set_cluster(i, 0);
            }
        }
    }

    //Memory allocation must be atomic
    end_no_ints(reenable);
}

pub unsafe fn realloc(ptr: usize, size: usize) -> usize {
    let mut ret = 0;

    //Memory allocation must be atomic
    let reenable = start_no_ints();

    if size == 0 {
        if ptr > 0 {
            unalloc(ptr);
        }
    }else{
        let old_size = alloc_size(ptr);
        if size <= old_size {
            ret = ptr;
        }else{
            ret = alloc(size);
            if ptr > 0 {
                if ret > 0 {
                    let copy_size = min(old_size, size);

                    ::memmove(ret as *mut u8, ptr as *const u8, copy_size);
                }
                unalloc(ptr);
            }
        }
    }

    end_no_ints(reenable);

    return ret;
}

pub unsafe fn realloc_inplace(ptr: usize, size: usize) -> usize {
    let old_size = alloc_size(ptr);
    if size <= old_size {
        return size;
    }else{
        return old_size;
    }
}

pub fn memory_used() -> usize{
    let mut ret = 0;
    unsafe{
        //Memory allocation must be atomic
        let reenable = start_no_ints();

        for i in 0..CLUSTER_COUNT {
            if cluster(i) != 0 && cluster(i) != 0xFFFFFFFF {
                ret += CLUSTER_SIZE;
            }
        }

        //Memory allocation must be atomic
        end_no_ints(reenable);
    }
    return ret;
}

pub fn memory_free() -> usize{
    let mut ret = 0;
    unsafe{
        //Memory allocation must be atomic
        let reenable = start_no_ints();

        for i in 0..CLUSTER_COUNT {
            if cluster(i) == 0 {
                ret += CLUSTER_SIZE;
            }
        }

        //Memory allocation must be atomic
        end_no_ints(reenable);
    }
    return ret;
}
