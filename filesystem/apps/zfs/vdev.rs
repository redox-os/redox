use redox::Vec;

use super::from_bytes::FromBytes;
use super::metaslab::{Metaslab, MetaslabGroup};
use super::nvpair::NvList;

#[repr(packed)]
pub struct VdevLabel {
    pub blank: [u8; 8 * 1024],
    pub boot_header: [u8; 8 * 1024],
    pub nv_pairs: [u8; 112 * 1024],
    pub uberblocks: [u8; 128 * 1024],
}

impl FromBytes for VdevLabel { }

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AllocType {
    Load = 0,
    Add,
    Spare,
    L2Cache,
    RootPool,
    Split,
    Attach,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Vdev {
    id: u64, // child number in vdev parent
    guid: u64, // unique ID for this vdev
    guid_sum: u64, // self guid + all child guids
    orig_guid: u64, // orig. guid prior to remove
    asize: u64, // allocatable device capacity
    min_asize: u64, // min acceptable asize
    max_asize: u64, // max acceptable asize
    ashift: u64, // block alignment shift

    // Top level only
    ms_array_object: u64,
    ms_group: MetaslabGroup,
    metaslabs: Vec<Metaslab>,

    // Leaf only
}

impl Vdev {
    pub fn new(nv: &NvList, alloc_type: AllocType) -> Self {
        Vdev {
            id: 0,
            guid: 0,
            guid_sum: 0,
            orig_guid: 0,
            asize: 0,
            min_asize: 0,
            max_asize: 0,
            ashift: 0,

            ms_array_object: 0,
            ms_group: MetaslabGroup,
            metaslabs: vec![],
        }
    }
}
