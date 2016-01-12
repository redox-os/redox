use std::cmp;
use std::rc::Rc;

use super::avl;
use super::dmu_objset::ObjectSet;
use super::space_map::{self, Segment, SpaceMap};
use super::taskq::{self, Taskq};
use super::txg;
use util;
use super::vdev;
use super::zfs;

// A metaslab class encompasses a category of allocatable top-level vdevs.
// Each top-level vdev is associated with a metaslab group which defines
// the allocatable region for that vdev. Examples of these categories include
// "normal" for data block allocations (i.e. main pool allocations) or "log"
// for allocations designated for intent log devices (i.e. slog devices).
// When a block allocation is requested from the SPA it is associated with a
// metaslab_class_t, and only top-level vdevs (i.e. metaslab groups) belonging
// to the class can be used to satisfy that request. Allocations are done
// by traversing the metaslab groups that are linked off of the `rotor` field.
// This rotor points to the next metaslab group where allocations will be
// attempted. Allocating a block is a 3 step process -- select the metaslab
// group, select the metaslab, and then allocate the block. The metaslab
// class defines the low-level block allocator that will be used as the
// final step in allocation. These allocators are pluggable allowing each class
// to use a block allocator that best suits that class.
//
pub struct MetaslabClass {
    // spa: *Spa,
    // rotor: *MetaslabGroup,
    ops: Rc<MetaslabOps>,
    aliquot: u64,
    alloc_groups: u64, // # of allocatable groups
    alloc: u64, // total allocated space
    deferred: u64, // total deferred frees
    space: u64, // total space (alloc + free)
    dspace: u64, /* total deflated space
                  * histogram: [u64, RANGE_TREE_HISTOGRAM_SIZE],
                  * fastwrite_lock: kmutex_t, */
}

impl MetaslabClass {
    pub fn create(ops: Rc<MetaslabOps>) -> MetaslabClass {
        // mutex_init(&mc->mc_fastwrite_lock, NULL, MUTEX_DEFAULT, NULL);

        MetaslabClass {
            // rotor: NULL,
            ops: ops,
            aliquot: 0,
            alloc_groups: 0,
            alloc: 0,
            deferred: 0,
            space: 0,
            dspace: 0,
        }
    }
}

// Metaslab groups encapsulate all the allocatable regions (i.e. metaslabs)
// of a top-level vdev. They are linked togther to form a circular linked
// list and can belong to only one metaslab class. Metaslab groups may become
// ineligible for allocations for a number of reasons such as limited free
// space, fragmentation, or going offline. When this happens the allocator will
// simply find the next metaslab group in the linked list and attempt
// to allocate from that group instead.
//
pub struct MetaslabGroup {
    // lock: kmutex_t,
    metaslab_tree: avl::Tree<MetaslabAvlNode, (u64, u64)>,
    aliquot: u64,
    allocatable: bool, // can we allocate?
    free_capacity: u64, // percentage free
    bias: i64,
    activation_count: i64,
    ms_class: Rc<MetaslabClass>,
    // vdev: vdev::TreeIndex,
    taskq: Taskq,
    // prev: *MetaslabGroup,
    // next: *MetaslabGroup,
    fragmentation: u64, // histogram: [u64; RANGE_TREE_HISTOGRAM_SIZE],
}

impl MetaslabGroup {
    pub fn create(ms_class: Rc<MetaslabClass>) -> Self {
        let metaslab_key = Rc::new(|ms: &MetaslabAvlNode| (ms.weight, ms.start));
        let taskq = Taskq::new("metaslab_group_taskq".to_string(),
                               // metaslab_load_pct
                               4,
                               10,
                               -1i64 as u64,
                               // TASKQ_THREADS_CPU_PCT | TASKQ_DYNAMIC
                               0);

        MetaslabGroup {
            // lock: kmutex_t,
            metaslab_tree: avl::Tree::new(metaslab_key),
            aliquot: 0,
            allocatable: false, // can we allocate?
            free_capacity: 0, // percentage free
            bias: 0,
            activation_count: 0,
            ms_class: ms_class,
            // vdev: vdev,
            taskq: taskq,
            // prev: *MetaslabGroup,
            // next: *MetaslabGroup,
            fragmentation: 0, // histogram: [0; RANGE_TREE_HISTOGRAM_SIZE],
        }
    }

    pub fn add(&mut self, index: usize, m: &Metaslab) {
        self.metaslab_tree.insert(MetaslabAvlNode {
            index: index,
            start: m.start,
            weight: m.weight,
        });
    }

    pub fn activate(&mut self) {
        // metaslab_class_t *mc = self.class;
        // metaslab_group_t *mgprev, *mgnext;
        //
        // assert!(spa_config_held(ms_class.spa, SCL_ALLOC, RW_WRITER));
        //
        // assert!(ms_class.rotor != mg);
        // assert!(self.prev == NULL);
        // assert!(self.next == NULL);
        // assert!(self.activation_count <= 0);
        //
        // if (++self.activation_count <= 0)
        // return;
        //
        // self.aliquot = metaslab_aliquot * cmp::max(1, self.vdev->vdev_children);
        // metaslab_group_alloc_update(mg);
        //
        // if (mgprev = ms_class.rotor) == NULL {
        // self.prev = mg;
        // self.next = mg;
        // } else {
        // mgnext = mgprev->mg_next;
        // self.prev = mgprev;
        // self.next = mgnext;
        // mgprev->mg_next = mg;
        // mgnext->mg_prev = mg;
        // }
        // ms_class.rotor = mg;
    }
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

// This value defines the number of elements in the lbas array. The value
// of 64 was chosen as it covers all power of 2 buckets up to UINT64_MAX.
// This is the equivalent of highbit(UINT64_MAX).
const MAX_LBAS: usize = 64;

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
//    free segment (tree) --------> alloc_tree ----> (write to space map)
//         ^
//         |
//         |                          free_tree <--- FREE
//         |                               |
//         |                               |
//         |                               |
//         +----------- defer_tree <-------+---------> (write to space map)
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
// needs to condense then we must set the condensing flag to ensure
// that allocations are not performed on the metaslab that is being written.
//

pub struct Metaslab {
    // lock: kmutex_t,
    // load_cv: kcondvar_t,
    space_map: Option<SpaceMap>,
    ops: Rc<MetaslabOps>,
    id: u64,
    start: u64,
    size: u64,
    fragmentation: u64,

    // Sorted by start
    alloc_tree: Vec<avl::Tree<space_map::Segment, u64>>, // txg::TXG_SIZE
    free_tree: Vec<avl::Tree<space_map::Segment, u64>>, // txg::TXG_SIZE
    defer_tree: Vec<avl::Tree<space_map::Segment, u64>>, // txg::DEFER_SIZE
    tree: avl::Tree<space_map::Segment, u64>,

    condensing: bool,
    condense_wanted: bool,
    loaded: bool,
    loading: bool,

    defer_space: i64, // sum of defermap[] space
    weight: u64, // weight vs others in group
    access_txg: u64,

    // The metaslab block allocators can optionally use a size-ordered
    // range tree and/or an array of LBAs. Not all allocators use
    // this functionality. The size_tree should always contain the
    // same number of segments as the tree. The only difference
    // is that the size_tree is ordered by segment sizes.
    size_tree: avl::Tree<space_map::Segment, u64>, // Sorted by size
    lbas: [u64; MAX_LBAS], /* group: *MetaslabGroup,
                            * avl_node_t ms_group_node, // node in metaslab group tree
                            * txg_node_t ms_txg_node, // per-txg dirty metaslab links */
}

impl Metaslab {
    pub fn new(ops: Rc<MetaslabOps>,
               id: u64,
               start: u64,
               size: u64,
               space_map: Option<SpaceMap>)
               -> Self {
        let seg_key_start = Rc::new(|seg: &Segment| seg.start);
        let seg_key_size = Rc::new(|seg: &Segment| seg.size);

        Metaslab {
            // lock: kmutex_t,
            // load_cv: kcondvar_t,
            space_map: space_map,
            ops: ops,
            id: id,
            start: start,
            size: size,
            fragmentation: 0,

            alloc_tree: (0..txg::TXG_SIZE).map(|x| avl::Tree::new(seg_key_start.clone())).collect(),
            free_tree: (0..txg::TXG_SIZE).map(|x| avl::Tree::new(seg_key_start.clone())).collect(),
            defer_tree: (0..txg::DEFER_SIZE)
                            .map(|x| avl::Tree::new(seg_key_start.clone()))
                            .collect(),
            tree: avl::Tree::new(seg_key_start),

            condensing: false,
            condense_wanted: false,
            loaded: false,
            loading: false,

            defer_space: 0,
            weight: 0,
            access_txg: 0,

            size_tree: avl::Tree::new(seg_key_size),
            lbas: [0; MAX_LBAS], /* group: *MetaslabGroup,
                                  * avl_node_t ms_group_node, // node in metaslab group tree
                                  * txg_node_t ms_txg_node, // per-txg dirty metaslab links */
        }
    }

    pub fn init(mos: &mut ObjectSet,
                vdev: &mut vdev::Vdev,
                id: u64,
                object: u64,
                txg: u64)
                -> zfs::Result<Self> {
        // We assume this is a top-level vdev
        let vdev_top = try!(vdev.top.as_mut().ok_or(zfs::Error::Invalid));

        // mutex_init(&ms.lock, NULL, MUTEX_DEFAULT, NULL);
        // cv_init(&ms->ms_load_cv, NULL, CV_DEFAULT, NULL);
        let start = id << vdev_top.ms_shift;
        let size = 1 << vdev_top.ms_shift;

        // We only open space map objects that already exist. All others
        // will be opened when we finally allocate an object for it.
        let space_map = if object != 0 {
            Some(try!(SpaceMap::open(mos,
                                     object,
                                     start,
                                     size,
                                     vdev.ashift as u8 /* , &ms.lock */)))
        } else {
            None
        };

        let mut metaslab = Self::new(vdev_top.ms_group.ms_class.ops.clone(),
                                     id,
                                     start,
                                     size,
                                     space_map);

        vdev_top.ms_group.add(id as usize, &metaslab);

        // metaslab.fragmentation = metaslab_fragmentation(metaslab);

        // If we're opening an existing pool (txg == 0) or creating
        // a new one (txg == TXG_INITIAL), all space is available now.
        // If we're adding space to an existing pool, the new space
        // does not become available until after this txg has synced.
        if txg <= txg::TXG_INITIAL as u64 {
            // metaslab_sync_done(metaslab, 0);
        }

        // If metaslab_debug_load is set and we're initializing a metaslab
        // that has an allocated space_map object then load the its space
        // map so that can verify frees.
        // if metaslab_debug_load && metaslab.space_map.is_some() {
        // try!(metaslab.load());
        // }


        // if txg != 0 {
        // vdev.dirty(0, NULL, txg);
        // vdev.dirty(vdev::DIRTY_METASLAB, ms, txg);
        // }

        Ok(metaslab)
    }

    pub fn load(&mut self) -> zfs::Result<()> {
        let mut result = Ok(());
        // assert!(MUTEX_HELD(&self.lock));
        assert!(!self.loaded);
        assert!(!self.loading);

        self.loading = true;

        // If the space map has not been allocated yet, then treat
        // all the space in the metaslab as free and add it to the
        // tree.
        if let Some(ref mut space_map) = self.space_map {
            // result = space_map.load(&mut self.tree, space_map::AllocType::Free);
        } else {
            self.tree.insert(Segment {
                start: self.start,
                size: self.size,
            });
        }

        self.loaded = result.is_ok();
        self.loading = false;

        if self.loaded {
            for t in 0..txg::DEFER_SIZE {
                // self.defer_tree[t].in_order(range_tree_remove, self.tree);
            }
        }
        // cv_broadcast(&self.load_cv);
        result
    }

    pub fn load_wait(&self) {
        while self.loading {
            assert!(!self.loaded);
            // cv_wait(&msp->ms_load_cv, &msp->ms_lock);
        }
    }

    fn activate(&mut self, activation_weight: u64) -> zfs::Result<()> {
        // TODO
        // assert!(MUTEX_HELD(&self.lock));
        //
        // if self.weight & METASLAB_ACTIVE_MASK == 0 {
        // self.load_wait();
        // if !self.loaded {
        // if let Err(e) = self.load() {
        // metaslab_group_sort(self.group, msp, 0);
        // return Err(e);
        // }
        // }
        //
        // metaslab_group_sort(self.group, self, self.weight | activation_weight);
        // }
        // assert!(self.loaded);
        // assert!(self.weight & METASLAB_ACTIVE_MASK);


        Ok(())
    }
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

pub struct MetaslabOps {
    pub alloc: fn(ms: &mut Metaslab, size: u64) -> u64,
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

// The first-fit block allocator
pub fn ff_alloc(ms: &mut Metaslab, size: u64) -> u64 {
    // Find the largest power of 2 block size that evenly divides the
    // requested size. This is used to try to allocate blocks with similar
    // alignment from the same area of the metaslab (i.e. same cursor
    // bucket) but it does not guarantee that other allocations sizes
    // may exist in the same region.
    let align = size & -(size as i64) as u64;
    let ref mut cursor = ms.lbas[(util::highbit64(align) - 1) as usize];
    let ref mut tree = ms.tree;

    // return metaslab_block_picker(tree, cursor, size, align);
    return 0;
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////
// This is a helper function that can be used by the allocator to find
// a suitable block to allocate. This will search the specified AVL
// tree looking for a block that matches the specified criteria.
// fn metaslab_block_picker(tree: &mut avl::Tree, cursor: &mut u64, size: u64, align: u64) -> u64 {
// range_seg_t *rs, rsearch;
// avl_index_t where;
//
// rsearch.rs_start = *cursor;
// rsearch.rs_end = *cursor + size;
//
// rs = tree.find(&rsearch, &where);
// if rs == NULL {
// rs = tree.nearest(where, AVL_AFTER);
// }
//
// while rs != NULL {
// let offset: u64 = util::p2roundup(rs->rs_start, align);
//
// if offset + size <= rs->rs_end {
// cursor = offset + size;
// return (offset);
// }
// rs = AVL_NEXT(t, rs);
// }
//
// If we know we've searched the whole map (*cursor == 0), give up.
// Otherwise, reset the cursor to the beginning and try again.
// if *cursor == 0 {
// return (-1ULL);
// }
//
// cursor = 0;
// return metaslab_block_picker(tree, cursor, size, align);
// }
/// /////////////////////////////////////////////////////////////////////////////////////////////////

struct MetaslabAvlNode {
    index: usize,
    weight: u64,
    start: u64,
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

// Allow allocations to switch to gang blocks quickly. We do this to
// avoid having to load lots of space_maps in a given txg. There are,
// however, some cases where we want to avoid "fast" ganging and instead
// we want to do an exhaustive search of all metaslabs on this device.
// Currently we don't allow any gang, slog, or dump device related allocations
// to "fast" gang.
// fn can_fast_gang(flags) -> bool {
// (flags) & (METASLAB_GANG_CHILD | METASLAB_GANG_HEADER | METASLAB_GANG_AVOID) == 0
// }


const METASLAB_WEIGHT_PRIMARY: u64 = 1 << 63;
const METASLAB_WEIGHT_SECONDARY: u64 = 1 << 62;
const METASLAB_ACTIVE_MASK: u64 = METASLAB_WEIGHT_PRIMARY | METASLAB_WEIGHT_SECONDARY;

// Metaslab granularity, in bytes. This is roughly similar to what would be
// referred to as the "stripe size" in traditional RAID arrays. In normal
// operation, we will try to write this amount of data to a top-level vdev
// before moving on to the next one.
static metaslab_aliquot: usize = 512 << 10;

// static metaslab_gang_bang: u64 = SPA_MAXBLOCKSIZE + 1;    /* force gang blocks */

// The in-core space map representation is more compact than its on-disk form.
// The zfs_condense_pct determines how much more compact the in-core
// space_map representation must be before we compact it on-disk.
// Values should be greater than or equal to 100.
static zfs_condense_pct: isize = 200;

// Condensing a metaslab is not guaranteed to actually reduce the amount of
// space used on disk. In particular, a space map uses data in increments of
// MAX(1 << ashift, space_map_blksz), so a metaslab might use the
// same number of blocks after condensing. Since the goal of condensing is to
// reduce the number of IOPs required to read the space map, we only want to
// condense when we can be sure we will reduce the number of blocks used by the
// space map. Unfortunately, we cannot precisely compute whether or not this is
// the case in metaslab_should_condense since we are holding ms_lock. Instead,
// we apply the following heuristic: do not condense a spacemap unless the
// uncondensed size consumes greater than zfs_metaslab_condense_block_threshold
// blocks.
static zfs_metaslab_condense_block_threshold: isize = 4;

// The zfs_mg_noalloc_threshold defines which metaslab groups should
// be eligible for allocation. The value is defined as a percentage of
// free space. Metaslab groups that have more free space than
// zfs_mg_noalloc_threshold are always eligible for allocations. Once
// a metaslab group's free space is less than or equal to the
// zfs_mg_noalloc_threshold the allocator will avoid allocating to that
// group unless all groups in the pool have reached zfs_mg_noalloc_threshold.
// Once all groups in the pool reach zfs_mg_noalloc_threshold then all
// groups are allowed to accept allocations. Gang blocks are always
// eligible to allocate on any metaslab group. The default value of 0 means
// no metaslab group will be excluded based on this criterion.
static zfs_mg_noalloc_threshold: isize = 0;

// Metaslab groups are considered eligible for allocations if their
// fragmenation metric (measured as a percentage) is less than or equal to
// zfs_mg_fragmentation_threshold. If a metaslab group exceeds this threshold
// then it will be skipped unless all metaslab groups within the metaslab
// class have also crossed this threshold.
static zfs_mg_fragmentation_threshold: isize = 85;

// Allow metaslabs to keep their active state as long as their fragmentation
// percentage is less than or equal to zfs_metaslab_fragmentation_threshold. An
// active metaslab that exceeds this threshold will no longer keep its active
// status allowing better metaslabs to be selected.
static zfs_metaslab_fragmentation_threshold: isize = 70;

// When set will load all metaslabs when pool is first opened.
static metaslab_debug_load: isize = 0;

// When set will prevent metaslabs from being unloaded.
static metaslab_debug_unload: isize = 0;

// Minimum size which forces the dynamic allocator to change
// it's allocation strategy.  Once the space map cannot satisfy
// an allocation of this size then it switches to using more
// aggressive strategy (i.e search by size rather than offset).
// static metaslab_df_alloc_threshold: u64 = SPA_MAXBLOCKSIZE;

// The minimum free space, in percent, which must be available
// in a space map to continue allocations in a first-fit fashion.
// Once the space_map's free space drops below this level we dynamically
// switch to using best-fit allocations.
static metaslab_df_free_pct: isize = 4;

// Percentage of all cpus that can be used by the metaslab taskq.
static metaslab_load_pct: isize = 50;

// Determines how many txgs a metaslab may remain loaded without having any
// allocations from it. As long as a metaslab continues to be used we will
// keep it loaded.
static metaslab_unload_delay: usize = txg::TXG_SIZE * 2;

// Max number of metaslabs per group to preload.
// static metaslab_preload_limit: isize = SPA_DVAS_PER_BP;

// Enable/disable preloading of metaslab.
static metaslab_preload_enabled: bool = true;

// Enable/disable fragmentation weighting on metaslabs.
static metaslab_fragmentation_factor_enabled: bool = true;

// Enable/disable lba weighting (i.e. outer tracks are given preference).
static metaslab_lba_weighting_enabled: bool = true;

// Enable/disable metaslab group biasing.
static metaslab_bias_enabled: bool = true;

// static uint64_t metaslab_fragmentation(metaslab_t *);
