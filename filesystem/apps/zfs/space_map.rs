use super::from_bytes::FromBytes;

const SPACE_MAP_HISTOGRAM_SIZE: usize = 32;

/// The `SpaceMapPhys` is the on-disk representation of the space map.
/// Consumers of space maps should never reference any of the members of this
/// structure directly. These members may only be updated in syncing context.
///
/// Note the smp_object is no longer used but remains in the structure
/// for backward compatibility.
///
/// The smp_histogram maintains a histogram of free regions. Each
/// bucket, smp_histogram[i], contains the number of free regions
/// whose size is:
/// 2^(i+sm_shift) <= size of free region in bytes < 2^(i+sm_shift+1)

#[derive(Debug)]
pub struct SpaceMapPhys {
    object: u64,   // on-disk space map object
    objsize: u64,  // size of the object
    alloc: u64,    // space allocated from the map
    //pad: [u64; 5], // reserved
    //histogram: [u64; SPACE_MAP_HISTOGRAM_SIZE],
}

impl FromBytes for SpaceMapPhys { }

struct Entry(u64);

impl Entry {
    pub fn size(&self) -> u64 {
        self.0 & 0x7F // 15 bits long
    }

    pub fn type(&self) -> u64 {
        (self.0 >> 15) & 0x1 // 1 bit long
    }

    pub fn offset(&self) -> u64 {
        (self.0 >> 16) & 0x7F // 47 bytes long
    }

    pub fn debug(&self) -> u64 {
        (self.0 >> 63) & 0x1 // 1 bit long
    }
}
