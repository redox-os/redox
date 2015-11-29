use std::{String, ToString, fmt};

use super::avl;
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

pub struct SpaceMap {
    pub size: usize,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MapType {
    Alloc = 0,
    Free = 1,
}

impl MapType {
    pub fn from_u64(u: u64) -> Option<Self> {
        match u {
            0 => Some(MapType::Alloc),
            1 => Some(MapType::Free),
            _ => None,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Entry(u64);

impl FromBytes for Entry { }

impl Entry {
    pub fn debug(&self) -> u64 {
        (self.0 >> 63) & 0x1 // 1 bit long
    }

    // Non-debug entries

    pub fn size(&self) -> u64 {
        self.0 & 0x7FFF // 15 bits long
    }

    pub fn map_type(&self) -> Option<MapType> {
        MapType::from_u64((self.0 >> 15) & 0x1) // 1 bit long
    }

    pub fn offset(&self) -> u64 {
        (self.0 >> 16) & 0x7FFFFFFFFFFF // 47 bytes long
    }

    // Debug entries

    pub fn action(&self) -> u64 {
        (self.0 >> 60) & 0x7 // 3 bits long
    }

    pub fn sync_pass(&self) -> u64 {
        (self.0 >> 50) & 0x3FF // 10 bits long
    }

    pub fn txg(&self) -> u64 {
        self.0 & 0x3FFFFFFFFFFFF // 50 bytes long
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.debug() == 1 {
            try!(write!(f, "DEBUG: action:0x{:X}  sync_pass:0x{:X}  txg:0x{:X}",
                   self.action(), self.sync_pass(), self.txg()));
        } else {
            try!(write!(f, "ENTRY: size:0x{:X}  map_type:0x{:?}  offset:0x{:X}",
                   self.size(), self.map_type(), self.offset()));
        }
        Ok(())
    }
}

pub fn load_space_map_avl(sm: &SpaceMap,
                          tree: &mut avl::Tree<Entry, u64>,
                          bytes: &[u8],
                          map_type: MapType) -> Result<(), String> {
    for i in 0..sm.size {
        let entry = Entry::from_bytes(&bytes[i*8..]).unwrap();
        let entry_map_type =
            match entry.map_type() {
                Some(map_type) => {
                    map_type
                },
                None => { return Err("Invalid map type".to_string()); },
            };
        if entry.debug() != 1 && entry_map_type == map_type {
            // it's not a debug entry and it's the right map type, add it to the tree
            tree.insert(entry);
        }
    }
    tree.in_order(|node| { println!("{:?}", node.value()); });
    
    Ok(())
}
