use std::{fmt, mem};

use super::avl;
use super::dmu_objset::ObjectSet;
use super::from_bytes::FromBytes;
use super::zfs;

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
    object: u64, // on-disk space map object
    objsize: u64, // size of the object
    alloc: u64, /* space allocated from the map
                 * pad: [u64; 5], // reserved
                 * histogram: [u64; SPACE_MAP_HISTOGRAM_SIZE], */
}

impl FromBytes for SpaceMapPhys {}

pub struct SpaceMap {
    start: u64, // start of map
    size: u64, // size of map
    shift: u8, // unit shift
    length: u64, // synced length
    alloc: u64, // synced space allocated
    // os: *ObjectSet,     // objset for this map
    object: u64, // object id for this map
    blksz: u32, // block size for space map
    // dbuf: *dmu_dbuf_t,   // space_map_phys_t dbuf
    phys: SpaceMapPhys, // on-disk space map
}

impl SpaceMap {
    /// Returns SpaceMapPhys, Dbuf, and block size
    // TODO
    // fn open_impl(os: &mut ObjectSet, object: u64) -> zfs::Result<(SpaceMapPhys, dmu::Dbuf, u64)> {
    // let dbuf = try!(dmu_bonus_hold(os, object, sm));
    //
    // let (block_size, num_blocks) = dmu_object_size_from_db(dbuf);
    // let phys = SpaceMapPhys::from_bytes(dbuf.data);
    //
    // Ok((phys, dbuf, block_size))
    // }


    pub fn open(os: &mut ObjectSet,
                object: u64,
                start: u64,
                size: u64,
                shift: u8)
                -> zfs::Result<Self> {
        assert!(object != 0);

        // TODO
        // let (phys, dbuf, block_size) = try!(Self::open_impl(os, object));
        let phys = SpaceMapPhys {
            object: 0, // on-disk space map object
            objsize: 0, // size of the object
            alloc: 0, // space allocated from the map
        };
        let block_size = 0;

        let mut space_map = SpaceMap {
            start: start,
            size: size,
            shift: shift,
            // os: os,
            object: object,
            length: 0,
            alloc: 0,
            blksz: block_size,
            // dbuf: dbuf,
            phys: phys,
        };

        Ok(space_map)
    }

    pub fn load_avl(&self,
                    tree: &mut avl::Tree<Segment, u64>,
                    bytes: &[u8],
                    map_type: MapType)
                    -> Result<(), String> {
        for i in 0..(self.size as usize) {
            let entry = Entry::from_bytes(&bytes[i * mem::size_of::<Entry>()..]).unwrap();
            let entry_map_type = match entry.map_type() {
                Some(map_type) => map_type,
                None => {
                    return Err("Invalid map type".to_string());
                }
            };
            if entry.debug() != 1 && entry_map_type == map_type {
                // it's not a debug entry and it's the right map type, add it to the tree
                tree.insert(Segment::from_entry(&entry));
            }
        }
        tree.in_order(|node| {
            println!("{:?}", node.value());
        });

        Ok(())
    }
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////
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

impl FromBytes for Entry {}

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
            try!(write!(f,
                        "DEBUG: action:0x{:X}  sync_pass:{:X}  txg:0x{:X}",
                        self.action(),
                        self.sync_pass(),
                        self.txg()));
        } else {
            try!(write!(f,
                        "ENTRY: size:0x{:X}  map_type:{:?}  offset:0x{:X}",
                        self.size(),
                        self.map_type(),
                        self.offset()));
        }
        Ok(())
    }
}


/// /////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct Segment {
    pub start: u64,
    pub size: u64,
}

impl Segment {
    fn from_entry(entry: &Entry) -> Self {
        Segment {
            start: entry.offset(),
            size: entry.size(),
        }
    }
}
