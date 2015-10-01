use core::ptr;

use common::memory::*;

pub unsafe fn set_page(virtual_address: usize, physical_address: usize) {
    let page = virtual_address / PAGE_SIZE;
    let table = page / PAGE_TABLE_SIZE;
    let entry = page % PAGE_TABLE_SIZE;
    let entry_address = PAGE_TABLES + (table * PAGE_TABLE_SIZE + entry) * 4;

    ptr::write(entry_address as *mut u32, (physical_address as u32 & 0xFFFFF000) | 1);

    asm!("invlpg [$0]"
        :
        : "{eax}"(virtual_address)
        : "memory"
        : "intel", "volatile");
}

pub unsafe fn missing_page(virtual_address: usize) {
    let page = virtual_address / PAGE_SIZE;
    let table = page / PAGE_TABLE_SIZE;
    let entry = page % PAGE_TABLE_SIZE;
    let entry_address = PAGE_TABLES + (table * PAGE_TABLE_SIZE + entry) * 4;

    ptr::write(entry_address as *mut u32, 0);

    asm!("invlpg [$0]"
        :
        : "{eax}"(virtual_address)
        : "memory"
        : "intel", "volatile");
}

pub unsafe fn identity_page(virtual_address: usize) {
    set_page(virtual_address, virtual_address);
}

pub unsafe fn page_bootstrap() {
    for table_i in 0..PAGE_TABLE_SIZE {
        ptr::write((PAGE_DIRECTORY + table_i * 4) as *mut u32, (PAGE_TABLES + table_i * PAGE_TABLE_SIZE * 4) as u32 | 1);

        for entry_i in 0..PAGE_TABLE_SIZE {
            let virtual_address = (table_i * PAGE_TABLE_SIZE + entry_i) * PAGE_SIZE;

            let page = virtual_address / PAGE_SIZE;
            let table = page / PAGE_TABLE_SIZE;
            let entry = page % PAGE_TABLE_SIZE;
            let entry_address = PAGE_TABLES + (table * PAGE_TABLE_SIZE + entry) * 4;

            ptr::write(entry_address as *mut u32, (virtual_address as u32 & 0xFFFFF000) | 1);
        }
    }

    asm!("mov cr3, $0\n
        mov $0, cr0\n
        or $0, 0x80000000\n
        mov cr0, $0\n"
        :
        : "{eax}"(PAGE_DIRECTORY)
        : "memory"
        : "intel", "volatile");
}

pub unsafe fn page_init() {
    for table_i in 0..PAGE_TABLE_SIZE {
        for entry_i in 0..PAGE_TABLE_SIZE {
            identity_page((table_i * PAGE_TABLE_SIZE + entry_i) * PAGE_SIZE);
        }
    }
    //Missing page to catch null pointer errors
    missing_page(0);
}
