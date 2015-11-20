use super::block_ptr::BlockPtr;
use super::from_bytes::FromBytes;

#[repr(packed)]
pub struct DslDatasetPhys {
    pub dir_obj: u64, // DMU_OT_DSL_DIR
    pub prev_snap_obj: u64, // DMU_OT_DSL_DATASET
    pub prev_snap_txg: u64,
    pub next_snap_obj: u64, // DMU_OT_DSL_DATASET
    pub snapnames_zapobj: u64, // DMU_OT_DSL_DS_SNAP_MAP 0 for snaps
    pub num_children: u64, // clone/snap children, ==0 for head
    pub creation_time: u64, // seconds since 1970
    pub creation_txg: u64,
    pub deadlist_obj: u64, // DMU_OT_DEADLIST
    //
    // ds_referenced_bytes, ds_compressed_bytes, and ds_uncompressed_bytes
    // include all blocks referenced by this dataset, including those
    // shared with any other datasets.
    //
    pub referenced_bytes: u64,
    pub compressed_bytes: u64,
    pub uncompressed_bytes: u64,
    pub unique_bytes: u64, // only relevant to snapshots
    //
    // The ds_fsid_guid is a 56-bit ID that can change to avoid
    // collisions.  The ds_guid is a 64-bit ID that will never
    // change, so there is a small probability that it will collide.
    //
    pub fsid_guid: u64,
    pub guid: u64,
    pub flags: u64, // DS_FLAG_*
    pub bp: BlockPtr,
    pub next_clones_obj: u64, // DMU_OT_DSL_CLONES
    pub props_obj: u64, // DMU_OT_DSL_PROPS for snaps
    pub userrefs_obj: u64, // DMU_OT_USERREFS
    pad: [u64; 5], // pad out to 320 bytes for good measure
}

impl FromBytes for DslDatasetPhys { }
