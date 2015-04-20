use common::debug::*;

const ALLOCATE_ADDRESS: u32 = 0x1000000;
const CLUSTER_COUNT: u32 = 64*1024; // 4 GiB
const CLUSTER_SIZE: u32 = 64*1024; // Of 64 K chunks

unsafe fn cluster(number: u32) -> u32{
    if number < CLUSTER_COUNT {
        return *((ALLOCATE_ADDRESS + number * 4) as *const u32);
    }else{
        return 0;
    }
}

unsafe fn set_cluster(number: u32, address: u32){
    if number < CLUSTER_COUNT {
        *((ALLOCATE_ADDRESS + number * 4) as *mut u32) = address;
    }
}

unsafe fn cluster_address(number: u32) -> u32{
    return ALLOCATE_ADDRESS + CLUSTER_COUNT*4 + number*CLUSTER_SIZE;
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

pub fn alloc(size: u32) -> u32{
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

pub fn unalloc(ptr: u32){
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