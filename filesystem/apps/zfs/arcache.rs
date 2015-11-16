use redox::{Vec, String, ToString};
use redox::collections::{BTreeMap, VecDeque};

use super::dvaddr::DVAddr;
use super::zio;

// Our implementation of the Adaptive Replacement Cache (ARC) is set up to allocate
// its buffer on the heap rather than in a private pool thing. This makes it much
// simpler to implement, but defers the fragmentation problem to the heap allocator.
// We named the type `ArCache` to avoid confusion with Rust's `Arc` reference type.
pub struct ArCache {
    // MRU
    // TODO: keep track of use counts. So mru_map becomes (use_count: u64, Vec<u8>)
    mru_map: BTreeMap<DVAddr, Vec<u8>>, // Most recently used cache
    mru_queue: VecDeque<DVAddr>, // Oldest DVAddrs are at the end
    mru_size: usize, // Max mru cache size in bytes
    mru_used: usize, // Used bytes in mru cache

    // MFU
    // TODO: Keep track of use counts. So mfu_map becomes (use_count: u64, Vec<u8>). Reset the use
    // count every once in a while. For instance, every 1000 reads. This will probably end up being
    // a knob for the user.
    mfu_map: BTreeMap<DVAddr, Vec<u8>>, // Most frequently used cache
    // TODO: Keep track of minimum frequency.
    mfu_min_freq: u64,
    mfu_size: usize, // Max mfu cache size in bytes
    mfu_used: usize, // Used bytes in mfu cache
}

impl ArCache {
    pub fn new() -> Self {
        ArCache {
            mru_map: BTreeMap::new(),
            mru_queue: VecDeque::new(),
            mru_size: 10,
            mru_used: 0,

            mfu_map: BTreeMap::new(),
            mfu_size: 10,
            mfu_used: 0,
        }
    }

    pub fn read(&mut self, reader: &mut zio::Reader, dva: &DVAddr) -> Result<Vec<u8>, String> {
        if let Some(block) = self.mru_map.get(dva) {
            // TODO: Keep track of MRU DVA use count. If it gets used a second time, move the block into
            // the MFU cache.

            // Block is cached
            return Ok(block.clone());
        }
        if let Some(block) = self.mfu_map.get(dva) {
            // TODO: keep track of DVA use count
            // Block is cached
            return Ok(block.clone());
        }

        // Block isn't cached, have to read it from disk
        let block = reader.read(dva.sector() as usize, dva.asize() as usize);
        self.cache_block(block);
    }

    fn mru_cache_block(&mut self, block: Vec<u8>) {
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

    // TODO: mfu_cache_block
}
