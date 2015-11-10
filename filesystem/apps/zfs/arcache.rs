use redox::Vec;
use redox::collections::{BTreeMap, VecDeque};

use super::dvaddr::DVAddr;
use super::zio;

// Our implementation of the Adaptive Replacement Cache (ARC) is set up to allocate
// its buffer on the heap rather than in a private pool thing. This makes it much
// simpler to implement, but defers the fragmentation problem to the heap allocator.
// We named the type `ArCache` to avoid confusion with Rust's `Arc` reference type.
pub struct ArCache {
    mru_map: BTreeMap<DVAddr, Vec<u8>>, // Most recently used cache
    mru_queue: VecDeque<DVAddr>, // Oldest paths are at the end
    mru_size: usize, // Max mru cache size in bytes
    mru_used: usize, // Used bytes in mru cache
}

impl ArCache {
    pub fn new() -> Self {
        ArCache {
            mru_map: BTreeMap::new(),
            mru_queue: VecDeque::new(),
            mru_size: 10,
            mru_used: 0,
        }
    }

    pub fn read(&mut self, reader: &mut zio::Reader, dva: &DVAddr) -> Vec<u8> {
        if let Some(block) = self.mru_map.get(dva) {
            // Block is cached
            return block.clone();
        }

        // Block isn't cached, have to read it from disk
        let block = reader.read(dva.sector() as usize, dva.asize() as usize);

        // If necessary, make room for the block in the cache
        if self.mru_used+block.len() > self.mru_size {
            // TODO: Evict oldest pages in mru cache until there is enough space for the
            // new block (IOW until `mru_used+block.len() <= mru_size`).
        }

        // Add the block to the cache
        self.mru_used += block.len();
        self.mru_map.insert(*dva, block);
        self.mru_queue.push_front(*dva);
        return self.mru_map.get(dva).unwrap().clone();
    }
}
