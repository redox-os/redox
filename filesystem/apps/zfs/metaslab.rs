use super::avl;
use super::space_map;

pub struct MetaslabClass;

pub struct MetaslabGroup;

/*
 * This value defines the number of elements in the ms_lbas array. The value
 * of 64 was chosen as it covers all power of 2 buckets up to UINT64_MAX.
 * This is the equivalent of highbit(UINT64_MAX).
 */
const MAX_LBAS: usize = 64;

const TXG_SIZE: usize = 4;
const TXG_DEFER_SIZE: usize = 2;

pub struct Metaslab {
    /*lock: kmutex_t,
    load_cv: kcondvar_t,
    sm: *space_map_t,
    ops: *metaslab_ops_t,*/
    id: u64,
    start: u64,
    size: u64,
    fragmentation: u64,

    alloctree: [avl::Tree<space_map::Entry, u64>; TXG_SIZE],
    freetree:  [avl::Tree<space_map::Entry, u64>; TXG_SIZE],
    defertree: [avl::Tree<space_map::Entry, u64>; TXG_DEFER_SIZE],
    tree:      avl::Tree<space_map::Entry, u64>,

    condensing: bool, // condensing?
    condense_wanted: bool,
    loaded: bool,
    loading: bool,

    deferspace: i64, // sum of ms_defermap[] space
    weight: u64, // weight vs. others in group
    access_txg: u64,

    /*
     * The metaslab block allocators can optionally use a size-ordered
     * range tree and/or an array of LBAs. Not all allocators use
     * this functionality. The ms_size_tree should always contain the
     * same number of segments as the ms_tree. The only difference
     * is that the ms_size_tree is ordered by segment sizes.
     */
    size_tree: avl::Tree<u64, u64>,
    lbas: [u64; MAX_LBAS],

    //group: *MetaslabGroup,
    //avl_node_t ms_group_node, // node in metaslab group tree
    //txg_node_t ms_txg_node, // per-txg dirty metaslab links
}
