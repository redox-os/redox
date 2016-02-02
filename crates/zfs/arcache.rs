use std::collections::{BTreeMap, VecDeque};

use super::dvaddr::DVAddr;
use super::zio;

/// MRU - Most Recently Used cache
struct Mru {
    map: BTreeMap<DVAddr, Vec<u8>>,
    queue: VecDeque<DVAddr>, // Oldest DVAddrs are at the end
    size: usize, // Max mru cache size in blocks
    used: usize, // Number of used blocks in mru cache
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

    pub fn cache_block(&mut self, dva: &DVAddr, block: Vec<u8>) -> Result<Vec<u8>, String> {
        // If necessary, make room for the block in the cache
        while self.used + (dva.asize() as usize) > self.size {
            let last_dva = match self.queue.pop_back() {
                Some(dva) => dva,
                None => return Err("No more ARC MRU items to free".to_string()),
            };
            self.map.remove(&last_dva);
            self.used -= last_dva.asize() as usize;
        }

        // Add the block to the cache
        self.used += dva.asize() as usize;
        self.map.insert(*dva, block);
        self.queue.push_front(*dva);
        Ok(self.map.get(dva).unwrap().clone())
    }
}

/// MFU - Most Frequently Used cache
struct Mfu {
    // TODO: Keep track of use counts. So mfu_map becomes (use_count: u64, Vec<u8>). Reset the use
    // count every once in a while. For instance, every 1000 reads. This will probably end up being
    // a knob for the user.
    // TODO: Keep track of minimum frequency and corresponding DVA
    map: BTreeMap<DVAddr, (u64, Vec<u8>)>,
    size: usize, // Max mfu cache size in blocks
    used: usize, // Number of used bytes in mfu cache
}

impl Mfu {
    pub fn new() -> Self {
        Mfu {
            map: BTreeMap::new(),
            size: 1000,
            used: 0,
        }
    }

    pub fn cache_block(&mut self, dva: &DVAddr, block: Vec<u8>) -> Result<Vec<u8>, String> {
        {
            let mut lowest_freq = ::std::u64::MAX;
            let mut lowest_dva: Result<DVAddr, String> = Err("No valid DVA found.".to_string());

            for (&dva_key, &(freq, _)) in self.map.iter() {
                if freq < lowest_freq {
                    lowest_freq = freq;
                    lowest_dva = Ok(dva_key);
                }
            }

            self.map.remove(&try!(lowest_dva));
        }

        // Add the block to the cache
        self.used += dva.asize() as usize;
        self.map.insert(*dva, (2, block));
        Ok(self.map.get(dva).unwrap().1.clone())
    }
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
        if let Some(block) = self.mru.map.remove(dva) {
            self.mfu.map.insert(*dva, (0, block.clone()));

            // Block is cached
            return Ok(block);
        }
        if let Some(block) = self.mfu.map.get_mut(dva) {
            // Block is cached
            if block.0 > 1000 {
                block.0 = 0;
            } else {
                block.0 += 1;
            }

            return Ok(block.1.clone());
        }

        // Block isn't cached, have to read it from disk
        let block = reader.read(dva.sector() as usize, dva.asize() as usize);

        // Blocks start in MRU cache
        self.mru.cache_block(dva, block)
    }
}
