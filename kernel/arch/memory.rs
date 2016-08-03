// TODO: Doc the rest

use core::{cmp, intrinsics, mem};
use core::ops::{Index, IndexMut};
use core::{ptr, slice};

use system::error::{Result, Error, ENOMEM};

use super::paging::{Page, PAGE_END};

pub const CLUSTER_ADDRESS: usize = PAGE_END;
pub const CLUSTER_COUNT: usize = 1024 * 1024; // 4 GiB
pub const CLUSTER_SIZE: usize = 4096; // Of 4 K chunks

pub const LOGICAL_OFFSET: usize = 0x80000000;

/// A wrapper around raw pointers
pub struct Memory<T> {
    ptr: *mut T,
    length: usize,
}

impl<T> Memory<T> {
    /// Allocate memory
    pub fn new(length: usize) -> Result<Self> {
        let alloc = unsafe { alloc(length * mem::size_of::<T>()) };
        if alloc > 0 {
            Ok(Memory {
                ptr: alloc as *mut T,
                length: length,
            })
        } else {
            Err(Error::new(ENOMEM))
        }
    }

    /// Allocate memory, aligned
    pub fn new_aligned(length: usize, align: usize) -> Result<Self> {
        let alloc = unsafe { alloc_aligned(length * mem::size_of::<T>(), align) };
        if alloc > 0 {
            Ok(Memory {
                ptr: alloc as *mut T,
                length: length,
            })
        } else {
            Err(Error::new(ENOMEM))
        }
    }

    /// Reallocate the memory
    pub fn renew(mut self, length: usize) -> Result<Self> {
        let alloc = unsafe { realloc(self.ptr as usize, length * mem::size_of::<T>()) };
        self.ptr = 0 as *mut T;
        if alloc > 0 {
            Ok(Memory {
                ptr: alloc as *mut T,
                length: length,
            })
        } else {
            Err(Error::new(ENOMEM))
        }
    }

    /// Reallocate the memory, aligned
    pub fn renew_aligned(mut self, length: usize, align: usize) -> Result<Self> {
        let alloc = unsafe { realloc_aligned(self.ptr as usize, length * mem::size_of::<T>(), align) };
        self.ptr = 0 as *mut T;
        if alloc > 0 {
            Ok(Memory {
                ptr: alloc as *mut T,
                length: length,
            })
        } else {
            Err(Error::new(ENOMEM))
        }
    }

    /// Get the length in T elements
    pub fn len(&self) -> usize {
        self.length
    }

    #[inline(always)]
    fn check_index(&self, i: usize) {
        if i >= self.length {
            panic!("Memory: {} >= {}", i, self.length);
        }
    }

    /// Read the memory
    pub fn read(&self, i: usize) -> T {
        self.check_index(i);
        unsafe { ptr::read(self.ptr.offset(i as isize)) }
    }

    /// Load the memory
    pub fn load(&self, i: usize) -> T {
        self.check_index(i);
        unsafe {
            intrinsics::atomic_singlethreadfence();
            ptr::read(self.ptr.offset(i as isize))
        }
    }

    /// Overwrite the memory
    pub fn write(&mut self, i: usize, value: T) {
        self.check_index(i);
        unsafe { ptr::write(self.ptr.offset(i as isize), value) };
    }

    /// Store the memory
    pub fn store(&mut self, i: usize, value: T) {
        self.check_index(i);
        unsafe {
            intrinsics::atomic_singlethreadfence();
            ptr::write(self.ptr.offset(i as isize), value)
        }
    }

    /// Get the address
    pub fn address(&self) -> usize {
        self.ptr as usize
    }

    /// Borrow as a slice
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr, self.length) }
    }

    /// Borrow as a mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.length) }
    }

    /// Borrow as a raw pointer
    pub unsafe fn as_ptr(&self) -> *const T {
        self.ptr as *const T
    }

    /// Borrow as a mutable raw pointer
    pub unsafe fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr as *mut T
    }

    /// Convert into a raw pointer
    pub unsafe fn into_raw(mut self) -> *mut T {
        let ptr = self.ptr;
        self.ptr = 0 as *mut T;
        ptr
    }
}

impl<T> Drop for Memory<T> {
    fn drop(&mut self) {
        if self.ptr as usize > 0 {
            unsafe { unalloc(self.ptr as usize) };
        }
    }
}

impl<T> Index<usize> for Memory<T> {
    type Output = T;

    fn index<'a>(&'a self, i: usize) -> &'a T {
        self.check_index(i);
        unsafe { &*self.ptr.offset(i as isize) }
    }
}

impl<T> IndexMut<usize> for Memory<T> {
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut T {
        self.check_index(i);
        unsafe { &mut *self.ptr.offset(i as isize) }
    }
}

/// A memory map entry
#[repr(packed)]
struct MemoryMapEntry {
    base: u64,
    len: u64,
    class: u32,
    acpi: u32,
}

const MEMORY_MAP: *const MemoryMapEntry = 0x500 as *const MemoryMapEntry;

/// Get the data (address) of a given cluster
pub unsafe fn cluster(number: usize) -> usize {
    if number < CLUSTER_COUNT {
        ptr::read((CLUSTER_ADDRESS + number * mem::size_of::<usize>()) as *const usize)
    } else {
        0
    }
}

/// Set the address of a cluster
pub unsafe fn set_cluster(number: usize, address: usize) {
    if number < CLUSTER_COUNT {
        ptr::write((CLUSTER_ADDRESS + number * mem::size_of::<usize>()) as *mut usize,
                   address);
    }
}

/// Convert an adress to the cluster number
pub unsafe fn address_to_cluster(address: usize) -> usize {
    if address >= CLUSTER_ADDRESS + CLUSTER_COUNT * mem::size_of::<usize>() {
        (address - CLUSTER_ADDRESS - CLUSTER_COUNT * mem::size_of::<usize>()) / CLUSTER_SIZE
    } else {
        0
    }
}

pub unsafe fn cluster_to_address(number: usize) -> usize {
    CLUSTER_ADDRESS + CLUSTER_COUNT * mem::size_of::<usize>() + number * CLUSTER_SIZE
}

/// Initialize clusters
pub unsafe fn cluster_init() {
    // First, set all clusters to the not present value
    for cluster in 0..CLUSTER_COUNT {
        set_cluster(cluster, 0xFFFFFFFF);
    }

    // Next, set all valid clusters to the free value
    // TODO: Optimize this function
    for i in 0..((0x5000 - 0x500) / mem::size_of::<MemoryMapEntry>()) {
        let entry = &*MEMORY_MAP.offset(i as isize);
        if entry.len > 0 && entry.class == 1 {
            for cluster in 0..CLUSTER_COUNT {
                let address = cluster_to_address(cluster);
                if address as u64 >= entry.base &&
                   (address as u64 + CLUSTER_SIZE as u64) <= (entry.base + entry.len) {
                    set_cluster(cluster, 0);
                }
            }
        }
    }
}

/// Allocate memory
pub unsafe fn alloc(size: usize) -> usize {
    alloc_aligned(size, 1)
}

/// Allocate memory, aligned
pub unsafe fn alloc_aligned(size: usize, align: usize) -> usize {
    if size > 0 {
        let mut number = 0;
        let mut count = 0;

        for i in 0..CLUSTER_COUNT {
            if cluster(i) == 0 && (count > 0 || cluster_to_address(i) % align == 0) {
                if count == 0 {
                    number = i;
                }

                count += 1;

                if count * CLUSTER_SIZE >= size {
                    break;
                }
            } else {
                count = 0;
            }
        }

        if count * CLUSTER_SIZE >= size {
            let address = cluster_to_address(number);

            for i in number..number + count {
                set_cluster(i, address);

                let cluster_address = cluster_to_address(i);

                let mut page = Page::new(cluster_address);
                let old = page.entry_data();
                page.map_kernel_write(cluster_address);

                ::memset(cluster_address as *mut u8, 0, CLUSTER_SIZE);

                page.set_entry_data(old);
                page.flush();
            }

            return address;
        }
    }

    0
}

/// Allocate a type
pub unsafe fn alloc_type<T>() -> *mut T {
    alloc(mem::size_of::<T>()) as *mut T
}

pub unsafe fn alloc_size(ptr: usize) -> usize {
    let mut size = 0;

    if ptr > 0 {
        for i in address_to_cluster(ptr)..CLUSTER_COUNT {
            if cluster(i) == ptr {
                size += CLUSTER_SIZE;
            } else {
                break;
            }
        }
    }

    size
}

pub unsafe fn unalloc(ptr: usize) {
    if ptr > 0 {
        for i in address_to_cluster(ptr)..CLUSTER_COUNT {
            if cluster(i) == ptr {
                set_cluster(i, 0);
            } else {
                break;
            }
        }
    }
}

pub unsafe fn unalloc_type<T>(ptr: *mut T) {
    unalloc(ptr as usize);
}


pub unsafe fn realloc(ptr: usize, size: usize) -> usize {
    realloc_aligned(ptr, size, 1)
}

pub unsafe fn realloc_aligned(ptr: usize, size: usize, align: usize) -> usize {
    let mut ret = 0;

    if size == 0 {
        if ptr > 0 {
            unalloc(ptr);
        }
    } else {
        let old_size = alloc_size(ptr);
        if size <= old_size {
            ret = ptr;
        } else {
            ret = alloc_aligned(size, align);
            if ptr > 0 {
                if ret > 0 {
                    let copy_size = cmp::min(old_size, size);

                    let read_cluster = address_to_cluster(ptr);
                    let write_cluster = address_to_cluster(ret);

                    for i in 0..(copy_size + CLUSTER_SIZE - 1)/CLUSTER_SIZE {
                        let read_address = cluster_to_address(read_cluster + i);
                        let write_address = cluster_to_address(write_cluster + i);

                        let mut read_page = Page::new(read_address);
                        let read_old = read_page.entry_data();
                        read_page.map_kernel_read(read_address);

                        let mut write_page = Page::new(write_address);
                        let write_old = write_page.entry_data();
                        write_page.map_kernel_write(write_address);

                        ::memmove(write_address as *mut u8, read_address as *const u8, CLUSTER_SIZE);

                        write_page.set_entry_data(write_old);
                        write_page.flush();

                        read_page.set_entry_data(read_old);
                        read_page.flush();
                    }
                }
                unalloc(ptr);
            }
        }
    }

    ret
}

pub unsafe fn realloc_inplace(ptr: usize, size: usize) -> usize {
    let old_size = alloc_size(ptr);
    if size <= old_size {
        size
    } else {
        old_size
    }
}

pub fn memory_used() -> usize {
    (0..CLUSTER_COUNT).fold(0usize, |ret, i| unsafe {
        if cluster(i) != 0 && cluster(i) != 0xFFFFFFFF {
            ret + CLUSTER_SIZE
        } else {
            ret
        }
    })
}

pub fn memory_free() -> usize {
    (0..CLUSTER_COUNT).fold(0usize, |ret, i| unsafe {
        if cluster(i) == 0 {
            ret + CLUSTER_SIZE
        } else {
            ret
        }
    })
}
