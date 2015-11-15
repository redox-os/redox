use redox::{Vec, String, ToString};
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

    pub fn read(&mut self, reader: &mut zio::Reader, dva: &DVAddr) -> Result<Vec<u8>, String> {
        if let Some(block) = self.mru_map.get(dva) {
            // Block is cached
            return Ok(block.clone());
        }

        // Block isn't cached, have to read it from disk
        let block = reader.read(dva.sector() as usize, dva.asize() as usize);

        // If necessary, make room for the block in the cache
        while self.mru_used + block.len() > self.mru_size {
            let last_dva = match self.mru_queue.pop_back()
            {
                Some(dva) => dva,
                None => return Err("No more ARC MRU items to free".to_string()),
            };
            self.mru_map.remove(&last_dva);
            self.mru_used -= last_dva.asize() as usize;
        }

        // Add the block to the cache
        self.mru_used += block.len();
        self.mru_map.insert(*dva, block);
        self.mru_queue.push_front(*dva);
        Ok(self.mru_map.get(dva).unwrap().clone())
    }
}
