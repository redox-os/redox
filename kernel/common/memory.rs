// TODO: Doc the rest
pub use common::heap::Memory;

use core::ops::{Index, IndexMut};
use core::{cmp, intrinsics, mem, ptr};

use scheduler;

use common::paging::PAGE_END;

// MT = Memory tree
// /// The offset adress for the memory map
// pub const MT_ADDR: usize = PAGE_END;
/// The depth of the memory tree
pub const MT_DEPTH: usize = 20;
/// The smallest possible memory block
pub const MT_ATOM: usize = 512;
/// The number of leafs in the memory tree
pub const MT_LEAFS: usize = 1 << MT_DEPTH;
/// The size of the root block
pub const MT_ROOT: usize = MT_LEAFS * MT_ATOM;
/// The number of nodes
pub const MT_NODES: usize = MT_LEAFS * 2 - 1;
/// The size of the memory map in bytes
pub const MT_BYTES: usize = MT_NODES / 4;
/// Empty memory tree
pub const MT: MemoryTree = MemoryTree {
    tree: StateTree { arr: StateArray { ptr: MT_PTR /* [0; MT_BYTES], */ } },
};
/// Where the heap starts
pub const HEAP_START: usize = PAGE_END + MT_BYTES;

/// The memory tree
pub const MT_PTR: usize = PAGE_END;

/// Ceil log 2
#[inline]
fn ceil_log2(n: usize) -> usize {
    floor_log2(n - 1) + 1
}

fn floor_log2(n: usize) -> usize {
    mem::size_of::<usize>() * 8 - n.leading_zeros() as usize
}


#[derive(Clone, Copy)]
/// The state of a memory block
pub enum MemoryState {
    /// None
    None = 3,
    /// Free
    Free = 0,
    /// Used
    Used = 1,
    /// Splitted
    Split = 2,
}

impl MemoryState {
    /// Convert an u8 to MemoryState
    pub fn from_u8(n: u8) -> MemoryState {
        match n {
            0 => MemoryState::Free,
            1 => MemoryState::Used,
            2 => MemoryState::Split,
            _ => MemoryState::None,
        }
    }
}

// #[derive(Clone, Copy)]
/// The memory tree
pub struct StateArray {
    /// The ptr to byte buffer of the state array
    ptr: usize, // bytes: [u8; MT_BYTES],
}

impl StateArray {
    /// Get the nth memory state (where n is a path in the tree)
    pub unsafe fn get(&self, n: usize) -> MemoryState {
        let byte = n / 4;
        let bit = 6 - 2 * (n % 4); // (from right)

        MemoryState::from_u8(((ptr::read((self.ptr + byte) as *mut u8) >> bit) & 0b11))
    }

    /// Set the nth memory state (where n is a path in the tree)
    pub unsafe fn set(&self, n: usize, val: MemoryState) {
        let byte = n / 4;
        let bit = 6 - 2 * (n % 4); // (from right)

        let ptr = (self.ptr + byte) as *mut u8;
        let b = ptr::read(ptr);

        ptr::write(ptr, ((val as u8) << bit) ^ (!(0b11 << bit) & b));
    }
}

// #[derive(Clone, Copy)]
pub struct StateTree {
    /// The array describing the state tree
    arr: StateArray,
}

impl StateTree {
    /// Get the position of a given node in the tree
    pub fn pos(&self, idx: usize, level: usize) -> usize {
        (Block {
            idx: idx,
            level: level,
        })
        .pos()
    }

    /// Set the value of a node
    pub unsafe fn set(&self, block: Block, state: MemoryState) {
        self.arr.set(block.pos(), state);
    }

    /// Get the value of a node
    pub unsafe fn get(&self, block: Block) -> MemoryState {
        self.arr.get(block.pos())
    }
}

#[derive(Clone, Copy)]
/// Sibling
pub enum Sibling {
    Left = 0,
    Right = 1,
}

#[derive(Clone, Copy)]
/// A memory block
pub struct Block {
    /// The index
    idx: usize,
    /// The level
    level: usize,
}

impl Block {
    /// Get the position of this block
    pub fn pos(&self) -> usize {
        self.idx + (1 << self.level) - 1
    }

    /// Get sibling side
    pub fn sibl(&self) -> Sibling {
        match self.idx & 1 {
            0 => Sibling::Left,
            _ => Sibling::Right,
        }
    }

    /// Get this blocks buddy
    pub fn get_buddy(&self) -> Block {
        Block {
            idx: self.idx ^ 1,
            level: self.level,
        }
    }

    /// The parrent of this block
    pub fn parrent(&self) -> Block {
        Block {
            idx: self.idx / 2,
            level: self.level.saturating_sub(1),
        }
    }

    /// The size of this block
    pub fn size(&self) -> usize {
        MT_ROOT / (1 << self.level)
    }

    /// Convert a pointer to a block
    pub fn from_ptr(ptr: usize) -> Block {
        // 47b4bbc7da718f45f89ce13d26a05ba89aa35510

        let pos = (ptr - HEAP_START) / MT_ATOM;
        let level = floor_log2(pos);

        let idx = (pos + 1) >> level;


        Block {
            level: level,
            idx: idx,
        }
    }

    /// Convert a block to a pointer
    pub fn to_ptr(&self) -> usize {
        HEAP_START + self.pos() * MT_ATOM
    }
}

// #[derive(Clone, Copy)]
/// Memory tree
pub struct MemoryTree {
    /// The state tree
    tree: StateTree,
}

impl MemoryTree {
    /// Split a block
    pub unsafe fn split(&self, block: Block) -> Block {
        self.tree.set(block, MemoryState::Split);

        let res = Block {
            idx: block.idx * 2,
            level: block.level + 1,
        };
        self.tree.set(res, MemoryState::Used);

        res
    }

    /// Allocate of minimum size, size
    pub unsafe fn alloc(&self, mut size: usize) -> Option<Block> {
        let order = ceil_log2(size / MT_ATOM) - 1;
        size = (1 << order) * MT_ATOM;
        let level = MT_DEPTH - order;

        let mut free = None;
        for i in 0..2 * (1 << level) {
            if let MemoryState::Free = self.tree.get(Block {
                level: level,
                idx: i,
            }) {
                free = Some(i);
            }
        }

        if let Some(n) = free {
            self.tree.set(Block {
                              level: level,
                              idx: n,
                          },
                          MemoryState::Used);

            Some(Block {
                idx: n,
                level: level,
            })
        } else {
            if level == 0 {
                None
            } else {
                // Kernel panic on OOM
                Some(if let Some(m) = self.alloc(size * 2) {
                    self.split(m)
                } else {
                    return None;
                })
            }
        }
    }

    /// Reallocate a block in an optimal way (by unifing it with its buddy)
    pub unsafe fn realloc(&self, mut block: Block, mut size: usize) -> Option<Block> {
        if let Sibling::Left = block.sibl() {
            let mut level = 0;

            let order = ceil_log2(size / MT_ATOM);
            size = (1 << order) * MT_ATOM;
            let level = MT_DEPTH - order;

            let delta = level as isize - block.level as isize;

            if delta < 0 {
                for i in 1..(-delta as usize) {
                    block = self.split(block);
                }

                Some(self.split(block))
            } else {
                let mut buddy = block.get_buddy();

                for i in 1..delta {

                    if let MemoryState::Free = self.tree.get(buddy) {
                    } else {
                        return None;
                    }

                    buddy = block.parrent().get_buddy();
                }

                if let MemoryState::Free = self.tree.get(buddy) {
                    let parrent = buddy.parrent();
                    self.tree.set(parrent, MemoryState::Used);
                    Some(parrent)
                } else {
                    None
                }

            }
        } else {
            None
        }

    }

    /// Deallocate a block
    pub unsafe fn dealloc(&mut self, block: Block) {
        self.tree.set(block, MemoryState::Free);
        if let MemoryState::Free = self.tree.get(block.get_buddy()) {
            self.tree.set(block.parrent(), MemoryState::Free);
        }
    }

}

/// Initialize dynamic memory (the heap)
pub fn memory_init() {
    unsafe {
        MT.tree.set(Block { level: 0, idx: 0 }, MemoryState::Free);
    }
}



/// Allocate memory
pub unsafe fn alloc(size: usize) -> usize {
    let ret;

    // Memory allocation must be atomic
    let reenable = scheduler::start_no_ints();

    //     if size > 0 {
    //         let mut number = 0;
    //         let mut count = 0;
    //
    //         for i in 0..CLUSTER_COUNT {
    //             if cluster(i) == 0 {
    //                 if count == 0 {
    //                     number = i;
    //                 }
    //                 count += 1;
    //                 if count * CLUSTER_SIZE > size {
    //                     break;
    //                 }
    //             } else {
    //                 count = 0;
    //             }
    //         }
    //         if count * CLUSTER_SIZE > size {
    //             let address = cluster_to_address(number);
    //
    //             ::memset(address as *mut u8, 0, count * CLUSTER_SIZE);
    //
    //             for i in number..number + count {
    //                 set_cluster(i, address);
    //             }
    //             ret = address;
    //         }
    //     }

    unsafe {
        // NOTE: In this case kernel panic is inevitable when OOM
        // TODO: Swap files
        ret = MT.alloc(size).unwrap().to_ptr();
    }

    // Memory allocation must be atomic
    scheduler::end_no_ints(reenable);

    ret
}

// TODO
pub unsafe fn alloc_aligned(size: usize, align: usize) -> usize {
    let ret;

    // Memory allocation must be atomic
    let reenable = scheduler::start_no_ints();

    //     if size > 0 {
    //         let mut number = 0;
    //         let mut count = 0;
    //
    //         for i in 0..CLUSTER_COUNT {
    //             if cluster(i) == 0 && (count > 0 || cluster_to_address(i) % align == 0) {
    //                 if count == 0 {
    //                     number = i;
    //                 }
    //                 count += 1;
    //                 if count * CLUSTER_SIZE > size {
    //                     break;
    //                 }
    //             } else {
    //                 count = 0;
    //             }
    //         }
    //         if count * CLUSTER_SIZE > size {
    //             let address = cluster_to_address(number);
    //
    //             ::memset(address as *mut u8, 0, count * CLUSTER_SIZE);
    //
    //             for i in number..number + count {
    //                 set_cluster(i, address);
    //             }
    //             ret = address;
    //         }
    //     }

    unsafe {
        // NOTE: In this case kernel panic is inevitable when OOM
        // TODO: Swap files
        ret = MT.alloc(size).unwrap().to_ptr();
    }

    // Memory allocation must be atomic
    scheduler::end_no_ints(reenable);

    ret
}

/// Allocate type
pub unsafe fn alloc_type<T>() -> *mut T {
    alloc(mem::size_of::<T>()) as *mut T
}

/// Get the allocate size
pub unsafe fn alloc_size(ptr: usize) -> usize {
    Block::from_ptr(ptr).size()
}

/// Unallocate
pub unsafe fn unalloc(ptr: usize) {
    // Memory allocation must be atomic
    let reenable = scheduler::start_no_ints();

    //     if ptr > 0 {
    //         for i in address_to_cluster(ptr)..CLUSTER_COUNT {
    //             if cluster(i) == ptr {
    //                 set_cluster(i, 0);
    //             } else {
    //                 break;
    //             }
    //         }
    //     }
    let b = Block::from_ptr(ptr);
    MT.dealloc(b);
    for i in 0..b.size() {
        ptr::write((ptr + i) as *mut u8, 0);
    }

    // Memory allocation must be atomic
    scheduler::end_no_ints(reenable);
}

/// Reallocate
pub unsafe fn realloc(ptr: usize, size: usize) -> usize {
    let ret;

    // Memory allocation must be atomic
    let reenable = scheduler::start_no_ints();

    if let Some(mut b) = MT.realloc(Block::from_ptr(ptr), size) {
        ret = b.to_ptr();
    } else {
        unalloc(ptr);
        ret = alloc(size);
        // TODO Optimize
        for n in 0..size {
            ptr::write((ret + n) as *mut u8, ptr::read((ptr + n) as *mut u8));
        }
    }


    scheduler::end_no_ints(reenable);

    ret
}

/// Reallocate in place
pub unsafe fn realloc_inplace(ptr: usize, size: usize) -> usize {
    let old_size = alloc_size(ptr);
    if size <= old_size {
        size
    } else {
        old_size
    }
}

pub fn memory_used() -> usize {
    let mut ret = 0;
    unsafe {
        // TODO

        // Memory allocation must be atomic
    }
    ret
}

pub fn memory_free() -> usize {
    let mut ret = 0;
    unsafe {
        // Memory allocation must be atomic
        // TODO

        // Memory allocation must be atomic
    }
    ret
}


// /// A memory map entry
// #[repr(packed)]
// struct MemoryMapEntry {
//     base: u64,
//     len: u64,
//     class: u32,
//     acpi: u32,
// }
//
// const MEMORY_MAP: *const MemoryMapEntry = 0x500 as *const MemoryMapEntry;
//
// /// Get the data (address) of a given cluster
// pub unsafe fn cluster(number: usize) -> usize {
//     if number < CLUSTER_COUNT {
//         ptr::read((CLUSTER_ADDRESS + number * mem::size_of::<usize>()) as *const usize)
//     } else {
//         0
//     }
// }
//
// /// Set the address of a cluster
// pub unsafe fn set_cluster(number: usize, address: usize) {
//     if number < CLUSTER_COUNT {
//         ptr::write((CLUSTER_ADDRESS + number * mem::size_of::<usize>()) as *mut usize,
//                    address);
//     }
// }
//
// /// Convert an adress to the cluster number
// pub unsafe fn address_to_cluster(address: usize) -> usize {
//     if address >= CLUSTER_ADDRESS + CLUSTER_COUNT * mem::size_of::<usize>() {
//         (address - CLUSTER_ADDRESS - CLUSTER_COUNT * mem::size_of::<usize>()) / CLUSTER_SIZE
//     } else {
//         0
//     }
// }
//
// pub unsafe fn cluster_to_address(number: usize) -> usize {
//     CLUSTER_ADDRESS + CLUSTER_COUNT * mem::size_of::<usize>() + number * CLUSTER_SIZE
// }
//
// /// Initialize clusters
// pub unsafe fn cluster_init() {
//     // First, set all clusters to the not present value
//     for cluster in 0..CLUSTER_COUNT {
//         set_cluster(cluster, 0xFFFFFFFF);
//     }
//
//     // Next, set all valid clusters to the free value
//     // TODO: Optimize this function
//     for i in 0..((0x5000 - 0x500) / mem::size_of::<MemoryMapEntry>()) {
//         let entry = &*MEMORY_MAP.offset(i as isize);
//         if entry.len > 0 && entry.class == 1 {
//             for cluster in 0..CLUSTER_COUNT {
//                 let address = cluster_to_address(cluster);
//                 if address as u64 >= entry.base &&
//                    (address as u64 + CLUSTER_SIZE as u64) <= (entry.base + entry.len) {
//                     set_cluster(cluster, 0);
//                 }
//             }
//         }
//     }
// }
