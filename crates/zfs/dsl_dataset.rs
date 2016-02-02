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
    // ds_referenced_bytes, ds_compressed_bytes, and ds_uncompressed_bytes
    // include all blocks referenced by this dataset, including those
    // shared with any other datasets.
    //
    pub referenced_bytes: u64,
    pub compressed_bytes: u64,
    pub uncompressed_bytes: u64,
    pub unique_bytes: u64, // only relevant to snapshots
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

impl FromBytes for DslDatasetPhys {}

//------------------------------------------------------------------------------------------------//

// struct DslDataset {
// dmu_buf_user_t ds_dbu,
//
// Immutable:
// dsl_dir *ds_dir,
// dmu_buf_t *ds_dbuf,
// object: u64,
// fsid_guid: u64,
// is_snapshot: bool,
//
// only used in syncing context, only valid for non-snapshots:
// dsl_dataset *ds_prev,
// bookmarks: u64,  // DMU_OTN_ZAP_METADATA
// large_blocks: bool,
// need_large_blocks: bool,
//
// has internal locking:
// dsl_deadlist_t ds_deadlist,
// bplist_t ds_pending_deadlist,
//
// protected by lock on pool's dp_dirty_datasets list
// txg_node_t ds_dirty_link,
// list_node_t ds_synced_link,
//
// ds_phys->ds_<accounting> is also protected by ds_lock.
// Protected by ds_lock:
// kmutex_t ds_lock,
// objset_t *ds_objset,
// ds_userrefs: u64,
// void *ds_owner,
//
// Long holds prevent the ds from being destroyed, they allow the
// ds to remain held even after dropping the dp_config_rwlock.
// Owning counts as a long hold.  See the comments above
// dsl_pool_hold() for details.
// refcount_t ds_longholds,
//
// no locking, only for making guesses
// ds_trysnap_txg: u64,
//
// for objset_open()
// kmutex_t ds_opening_lock,
//
// ds_reserved: u64,	// cached refreservation
// ds_quota: u64,	// cached refquota
//
// kmutex_t ds_sendstream_lock,
// list_t ds_sendstreams,
//
// Protected by ds_lock, keep at end of struct for better locality
// char ds_snapname[MAXNAMELEN],
// }
