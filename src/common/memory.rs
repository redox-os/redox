use core::mem::size_of;

use common::debug::*;

const ALLOCATE_ADDRESS: usize = 0x1000000;
const CLUSTER_COUNT: usize = 64*1024; // 4 GiB
const CLUSTER_SIZE: usize = 64*1024; // Of 64 K chunks

unsafe fn cluster(number: usize) -> usize{
    if number < CLUSTER_COUNT {
        return *((ALLOCATE_ADDRESS + number * size_of::<usize>()) as *const usize);
    }else{
        return 0;
    }
}

unsafe fn set_cluster(number: usize, address: usize){
    if number < CLUSTER_COUNT {
        *((ALLOCATE_ADDRESS + number * size_of::<usize>()) as *mut usize) = address;
    }
}

unsafe fn cluster_address(number: usize) -> usize{
    return ALLOCATE_ADDRESS + CLUSTER_COUNT * size_of::<usize>() + number*CLUSTER_SIZE;
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