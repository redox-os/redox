use redox::{Vec, String, ToString};
use redox::collections::{BTreeMap, VecDeque};

use super::dvaddr::DVAddr;
use super::zio;

/// MRU - Most Recently Used cache
struct Mru {
    // TODO: keep track of use counts. So mru_map becomes (use_count: u64, Vec<u8>)
    map: BTreeMap<DVAddr, (u64, Vec<u8>)>,
    queue: VecDeque<DVAddr>, // Oldest DVAddrs are at the end
    size: usize, // Max mru cache size in bytes
    used: usize, // Used bytes in mru cache
}

impl Mru {
    pub fn new() -> Self {
        Mru {
            map: BTreeMap::new(),
            queue: VecDeque::new(),
            size: 1000,
            used: 0,
        }
    }

    fn cache_block(&mut self, dva: &DVAddr, block: Vec<u8>) -> Result<Vec<u8>, String> {
        // If necessary, make room for the block in the cache
        while self.used + block.len() > self.size {
            let last_dva = match self.queue.pop_back() {
                Some(dva) => dva,
                None => return Err("No more ARC MRU items to free".to_string()),
            };
            self.map.remove(&last_dva);
            self.used -= last_dva.asize() as usize;
        }

        // Add the block to the cache
        self.used += block.len();
        self.map.insert(*dva, (0, block));
        self.queue.push_front(*dva);
        Ok(self.map.get(dva).unwrap().1.clone())
    }
}

/// MFU - Most Frequently Used cache
struct Mfu {
    // TODO: Keep track of use counts. So mfu_map becomes (use_count: u64, Vec<u8>). Reset the use
    // count every once in a while. For instance, every 1000 reads. This will probably end up being
    // a knob for the user.
    // TODO: Keep track of minimum frequency and corresponding DVA
    map: BTreeMap<DVAddr, (u64, Vec<u8>)>,
    size: usize, // Max mfu cache size in bytes
    used: usize, // Used bytes in mfu cache
}

impl Mfu {
    pub fn new() -> Self {
        Mfu {
            map: BTreeMap::new(),
            size: 1000,
            used: 0,
        }
    }

    // TODO: cache_block. Remove the DVA with the lowest frequency
    /*
    fn cache_block(&mut self, dva: &DVAddr, block: Vec<u8>) -> Result<Vec<u8>, String> {
    }
    */
}

// Our implementation of the Adaptive Replacement Cache (ARC) is set up to allocate
// its buffer on the heap rather than in a private pool thing. This makes it much
// simpler to implement, but defers the fragmentation problem to the heap allocator.
// We named the type `ArCache` to avoid confusion with Rust's `Arc` reference type.
pub struct ArCache {
    mru: Mru,
    mfu: Mfu,
}

impl ArCache {
    pub fn new() -> Self {
        ArCache {
            mru: Mru::new(),
            mfu: Mfu::new(),
        }
    }

    pub fn read(&mut self, reader: &mut zio::Reader, dva: &DVAddr) -> Result<Vec<u8>, String> {
        if let Some(block) = self.mru.map.get_mut(dva) {
            // TODO: Keep track of MRU DVA use count. If it gets used a second time, move the block into
            // the MFU cache.

            block.0 += 1;

            // Block is cached
            return Ok(block.1.clone());
        }
        if let Some(block) = self.mfu.map.get_mut(dva) {
            // TODO: keep track of DVA use count
            // Block is cached

            block.0 += 1;

            return Ok(block.1.clone());
        }

        // Block isn't cached, have to read it from disk
        let block = reader.read(dva.sector() as usize, dva.asize() as usize);

        // Blocks start in MRU cache
        self.mru.cache_block(dva, block)
    }
}
