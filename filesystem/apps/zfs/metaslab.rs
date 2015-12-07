use super::avl;
use super::space_map;

pub struct MetaslabClass;

pub struct MetaslabGroup;

//
// This value defines the number of elements in the ms_lbas array. The value
// of 64 was chosen as it covers all power of 2 buckets up to UINT64_MAX.
// This is the equivalent of highbit(UINT64_MAX).
//
const MAX_LBAS: usize = 64;

const TXG_SIZE: usize = 4;
const TXG_DEFER_SIZE: usize = 2;

//
// Each metaslab maintains a set of in-core trees to track metaslab operations.
// The in-core free tree (ms_tree) contains the current list of free segments.
// As blocks are allocated, the allocated segment are removed from the ms_tree
// and added to a per txg allocation tree (ms_alloctree). As blocks are freed,
// they are added to the per txg free tree (ms_freetree). These per txg
// trees allow us to process all allocations and frees in syncing context
// where it is safe to update the on-disk space maps. One additional in-core
// tree is maintained to track deferred frees (ms_defertree). Once a block
// is freed it will move from the ms_freetree to the ms_defertree. A deferred
// free means that a block has been freed but cannot be used by the pool
// until TXG_DEFER_SIZE transactions groups later. For example, a block
// that is freed in txg 50 will not be available for reallocation until
// txg 52 (50 + TXG_DEFER_SIZE).  This provides a safety net for uberblock
// rollback. A pool could be safely rolled back TXG_DEFERS_SIZE
// transactions groups and ensure that no block has been reallocated.
//
// The simplified transition diagram looks like this:
//
//
//      ALLOCATE
//         |
//         V
//    free segment (ms_tree) --------> ms_alloctree ----> (write to space map)
//         ^
//         |
//         |                           ms_freetree <--- FREE
//         |                                 |
//         |                                 |
//         |                                 |
//         +----------- ms_defertree <-------+---------> (write to space map)
//
//
// Each metaslab's space is tracked in a single space map in the MOS,
// which is only updated in syncing context. Each time we sync a txg,
// we append the allocs and frees from that txg to the space map.
// The pool space is only updated once all metaslabs have finished syncing.
//
// To load the in-core free tree we read the space map from disk.
// This object contains a series of alloc and free records that are
// combined to make up the list of all free segments in this metaslab. These
// segments are represented in-core by the ms_tree and are stored in an
// AVL tree.
//
// As the space map grows (as a result of the appends) it will
// eventually become space-inefficient. When the metaslab's in-core free tree
// is zfs_condense_pct/100 times the size of the minimal on-disk
// representation, we rewrite it in its minimized form. If a metaslab
// needs to condense then we must set the ms_condensing flag to ensure
// that allocations are not performed on the metaslab that is being written.
//

pub struct Metaslab {
    // lock: kmutex_t,
    // load_cv: kcondvar_t,
    // sm: *space_map_t,
    // ops: *metaslab_ops_t,
    id: u64,
    start: u64,
    size: u64,
    fragmentation: u64,

    alloctree: [avl::Tree<space_map::Entry, u64>; TXG_SIZE],
    freetree: [avl::Tree<space_map::Entry, u64>; TXG_SIZE],
    defertree: [avl::Tree<space_map::Entry, u64>; TXG_DEFER_SIZE],
    tree: avl::Tree<space_map::Entry, u64>,

    condensing: bool, // condensing?
    condense_wanted: bool,
    loaded: bool,
    loading: bool,

    deferspace: i64, // sum of ms_defermap[] space
    weight: u64, // weight vs. others in group
    access_txg: u64,

    //
    // The metaslab block allocators can optionally use a size-ordered
    // range tree and/or an array of LBAs. Not all allocators use
    // this functionality. The ms_size_tree should always contain the
    // same number of segments as the ms_tree. The only difference
    // is that the ms_size_tree is ordered by segment sizes.
    //
    size_tree: avl::Tree<u64, u64>,
    lbas: [u64; MAX_LBAS], /* group: *MetaslabGroup,
                            * avl_node_t ms_group_node, // node in metaslab group tree
                            * txg_node_t ms_txg_node, // per-txg dirty metaslab links */
}
