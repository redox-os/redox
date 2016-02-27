use super::from_bytes::FromBytes;

const DD_USED_NUM: usize = 5; // The number of variants in DslDirUsed

pub enum DslDirUsed {
    Head = 0,
    Snap,
    Child,
    ChildReserve,
    RefReserve,
}

#[repr(packed)]
pub struct DslDirPhys {
    pub creation_time: u64, // not actually used
    pub head_dataset_obj: u64,
    pub parent_obj: u64,
    pub origin_obj: u64,
    pub child_dir_zapobj: u64,
    // how much space our children are accounting for, for leaf
    // datasets, == physical space used by fs + snaps
    pub used_bytes: u64,
    pub compressed_bytes: u64,
    pub uncompressed_bytes: u64,
    // Administrative quota setting
    pub quota: u64,
    // Administrative reservation setting
    pub reserved: u64,
    pub props_zapobj: u64,
    pub deleg_zapobj: u64, // dataset delegation permissions
    pub flags: u64,
    pub used_breakdown: [u64; DD_USED_NUM],
    pub clones: u64, // dsl_dir objects
    pub pad: [u64; 13], // pad out to 256 bytes for good measure
}

impl FromBytes for DslDirPhys {}
