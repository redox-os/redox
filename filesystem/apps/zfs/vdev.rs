use super::from_bytes::FromBytes;

#[repr(packed)]
pub struct VdevLabel {
    pub blank: [u8; 8 * 1024],
    pub boot_header: [u8; 8 * 1024],
    pub nv_pairs: [u8; 112 * 1024],
    pub uberblocks: [u8; 128 * 1024],
}

impl FromBytes for VdevLabel { }

/// /////////////////////////////////////////////////////////////////////////////////////////////////

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
    ms_array: u64, // Leaf only
}
