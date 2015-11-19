use redox::Box;

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

pub enum MapType {
    Alloc = 0,
    Free = 1,
}

#[derive(Copy, Clone, Debug)]
pub struct Entry(u64);

impl FromBytes for Entry { }

impl Entry {
    fn debug(&self) -> u64 {
        (self.0 >> 63) & 0x1 // 1 bit long
    }

    // Non-debug entries

    fn size(&self) -> u64 {
        self.0 & 0x7FFF // 15 bits long
    }

    fn map_type(&self) -> u64 {
        (self.0 >> 15) & 0x1 // 1 bit long
    }

    fn offset(&self) -> u64 {
        (self.0 >> 16) & 0x7FFFFFFFFFFF // 47 bytes long
    }

    // Debug entries

    fn action(&self) -> u64 {
        (self.0 >> 60) & 0x7 // 3 bits long
    }

    fn sync_pass(&self) -> u64 {
        (self.0 >> 50) & 0x3FF // 10 bits long
    }

    fn txg(&self) -> u64 {
        self.0 & 0x3FFFFFFFFFFFF // 50 bytes long
    }
}

pub fn load_space_map_avl(sm: &SpaceMap, bytes: &[u8]) {
    let mut avl_tree = avl::Tree::new(Box::new(|x| *x));
    avl_tree.insert(1u64);
    avl_tree.insert(10);
    avl_tree.insert(6);
    avl_tree.insert(4);
    avl_tree.insert(8);
    avl_tree.insert(9);
    avl_tree.insert(3);
    avl_tree.in_order(|node| { println!("{}", node.value()); });
    for i in 0..sm.size {
        let entry = Entry::from_bytes(&bytes[i*8..]).unwrap();
        if entry.debug() == 1 {
            println!("DEBUG: action:0x{:X}  sync_pass:0x{:X}  txg:0x{:X}",
                     entry.action(), entry.sync_pass(), entry.txg());
        } else {
            println!("ENTRY: size:0x{:X}  map_type:0x{:X}  offset:0x{:X}",
                     entry.size(), entry.map_type(), entry.offset());
        }
    }
}
