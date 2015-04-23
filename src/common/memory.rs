use core::mem::size_of;

use common::debug::*;

const PAGE_DIRECTORY: usize = 0x1000000;
const PAGE_TABLE_SIZE: usize = 1024;
const PAGE_TABLES: usize = PAGE_DIRECTORY + PAGE_TABLE_SIZE * 4;

pub unsafe fn set_page(virtual_address: usize, physical_address: usize){
    let page = virtual_address / 4096;
    let table = page / 1024;
    let entry = page % 1024;
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
            identity_page((table_i * PAGE_TABLE_SIZE + entry_i) * 4096);
        }
    }
    
    asm!("mov cr3, $0\n
        mov $0, cr0\n
        or $0, 0x80000000\n
        mov cr0, $0\n"
        : : "{eax}"(PAGE_DIRECTORY) : : "intel");
}

const CLUSTER_ADDRESS: usize = 0x2000000;
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

pub fn cluster_init(){
    unsafe {
        d("TODO: Automatic memory detection, ");
        //CLUSTER_COUNT = 64*1024;
        dd(CLUSTER_COUNT);
        d(" Clusters, ");
        dd(CLUSTER_SIZE);
        d(" Bytes each");
        dl();
        
        for i in 0..CLUSTER_COUNT {
            set_cluster(i, 0);
        }
    }
}

pub fn alloc(size: usize) -> usize{
    unsafe{
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
    }
}

pub fn unalloc(ptr: usize){
    unsafe{
        if ptr > 0 {
            for i in 0..CLUSTER_COUNT {
                if cluster(i) == ptr {
                    set_cluster(i, 0);
                }
            }
        }
    }
}