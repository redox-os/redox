use core::cmp::min;
use core::mem::size_of;

const PAGE_DIRECTORY: usize = 0x300000;
const PAGE_TABLE_SIZE: usize = 1024;
const PAGE_TABLES: usize = PAGE_DIRECTORY + PAGE_TABLE_SIZE * 4;
const PAGE_SIZE: usize = 4*1024;

pub unsafe fn set_page(virtual_address: usize, physical_address: usize){
    let page = virtual_address / PAGE_SIZE;
    let table = page / PAGE_TABLE_SIZE;
    let entry = page % PAGE_TABLE_SIZE;
    let entry_address = PAGE_TABLES + (table * PAGE_TABLE_SIZE + entry) * 4;

    *(entry_address as *mut u32) = (physical_address as u32 & 0xFFFFF000) | 1;

    asm!("invlpg [$0]" : : "{eax}"(virtual_address) : : "intel");
}

pub unsafe fn identity_page(virtual_address: usize){
    set_page(virtual_address, virtual_address);
}

pub unsafe fn page_init(){
    for table_i in 0..PAGE_TABLE_SIZE {
        *((PAGE_DIRECTORY + table_i * 4) as *mut u32) = (PAGE_TABLES + table_i * PAGE_TABLE_SIZE * 4) as u32 | 1;

        for entry_i in 0..PAGE_TABLE_SIZE {
            identity_page((table_i * PAGE_TABLE_SIZE + entry_i) * PAGE_SIZE);
        }
    }

    asm!("mov cr3, $0\n
        mov $0, cr0\n
        or $0, 0x80000000\n
        mov cr0, $0\n"
        : : "{eax}"(PAGE_DIRECTORY) : : "intel");
}

const CLUSTER_ADDRESS: usize = PAGE_TABLES + PAGE_TABLE_SIZE * PAGE_TABLE_SIZE * 4 ;
const CLUSTER_COUNT: usize = 1024*1024; // 4 GiB
const CLUSTER_SIZE: usize = 4*1024; // Of 4 K chunks

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

unsafe fn cluster_address(number: usize) -> usize{
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
            let address = cluster_address(number);
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
        for i in 0..CLUSTER_COUNT {
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

pub unsafe fn unalloc(ptr: usize){
    if ptr > 0 {
        for i in 0..CLUSTER_COUNT {
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
