use std::{cmp, mem};
use std::rc::Rc;

use super::dmu_objset::ObjectSet;
use super::from_bytes::FromBytes;
use super::metaslab::{Metaslab, MetaslabClass, MetaslabGroup};
use super::nvpair::{NvList, NvValue};
use super::uberblock;
use super::util;
use super::vdev_file::VdevFile;
use super::zfs;

#[repr(packed)]
pub struct VdevLabel {
    pub blank: [u8; 8 * 1024],
    pub boot_header: [u8; 8 * 1024],
    pub nv_pairs: [u8; 112 * 1024],
    pub uberblocks: [u8; 128 * 1024],
}

impl FromBytes for VdevLabel {}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

pub trait IVdevOps {
    /// Returns (size, max_size, ashift)
    fn open(&mut self, vdev: &mut Vdev) -> zfs::Result<(u64, u64, u64)>;

    fn close(&mut self, vdev: &mut Vdev);

    /// Default asize function: return the MAX of psize with the asize of all children.  This is
    /// what's used by anything other than RAID-Z.
    fn asize(&mut self, vdev: &mut Vdev, psize: u64) -> u64;

    fn hold(&mut self, vdev: &mut Vdev);

    fn release(&mut self, vdev: &mut Vdev);
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

pub struct VdevOps {
    pub ops: Box<IVdevOps>,
    // io_start: fn(&zio::Zio),
    // io_done: fn(&zio::Zio),
    // state_change: fn(),
    vdev_type: String,
    is_leaf: bool,
}

impl VdevOps {
    pub fn vdev_type(&self) -> &str {
        self.vdev_type.as_ref()
    }
    pub fn is_leaf(&self) -> bool {
        self.is_leaf
    }
}

fn load_ops(vdev_type: &str, nv: &NvList) -> zfs::Result<VdevOps> {
    match vdev_type {
        "disk" => {
            Ok(VdevOps {
                ops: Box::new(try!(VdevFile::load(nv))),
                vdev_type: "disk".to_string(),
                is_leaf: true,
            })
        }
        _ => Err(zfs::Error::Invalid),
    }
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////
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

/// /////////////////////////////////////////////////////////////////////////////////////////////////

/// States are ordered from least to most healthy.
/// Vdevs `CannotOpen` and worse are considered unusable.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Unknown, // Uninitialized vdev
    Closed, // Not currently open
    Offline, // Not allowed to open
    Removed, // Explicitly removed from the system
    CannotOpen, // Tried top open, but failed
    Faulted, // External request to fault device
    Degraded, // Replicated vdev with unhealthy kids
    Healthy, // Presumed good
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

// Stuff that only top level vdevs have
pub struct Top {
    pub ms_array: u64, // object ID of metaslab array in MOS
    pub ms_shift: u64, // metaslab shift
    pub ms_group: MetaslabGroup, // metaslab group
    pub metaslabs: Vec<Metaslab>, // in-memory metaslab array
    pub is_hole: bool,
    pub removing: bool, // device is being removed?
}

impl Top {
    pub fn new(ms_array: u64, ms_shift: u64, ms_group: MetaslabGroup) -> Self {
        Top {
            ms_array: ms_array,
            ms_shift: ms_shift,
            ms_group: ms_group,
            metaslabs: vec![],
            is_hole: false, // TODO: zol checks vdev_ops for this, but idk what to do yet
            removing: false,
        }
    }
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Leaf {
    whole_disk: u64,
}

impl Leaf {
    pub fn new() -> Self {
        Leaf { whole_disk: 0 }
    }
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

// Note that a vdev can be a top-level, a leaf, both, or neither
pub struct Vdev {
    id: u64, // child number in vdev parent
    guid: u64, // unique ID for this vdev
    guid_sum: u64, // self guid + all child guids
    orig_guid: u64, // orig. guid prior to remove
    asize: u64, // allocatable device capacity
    min_asize: u64, // min acceptable asize
    max_asize: u64, // max acceptable asize
    pub ashift: u64, // block alignment shift
    state: State,
    prev_state: State,
    pub ops: VdevOps,
    parent: Option<TreeIndex>,
    top_vdev: Option<TreeIndex>,
    children: Vec<TreeIndex>,
    create_txg: u64, // txg when top-level was added

    pub top: Option<Top>,
    pub leaf: Option<Leaf>,
}

impl Vdev {
    pub fn new(id: u64,
               guid: Option<u64>,
               ashift: u64,
               ops: VdevOps,
               create_txg: u64,
               vdev_top: Option<Top>)
               -> Self {
        let guid = guid.unwrap_or_else(|| {
            // TODO: generate a guid
            0
        });

        // TODO vdev_queue_init

        Vdev {
            id: id,
            guid: guid,
            guid_sum: guid, // No children yet, so guid_sum is just my guid
            orig_guid: 0,
            asize: 0,
            min_asize: 0,
            max_asize: 0,
            ashift: ashift,
            state: State::Closed,
            prev_state: State::Unknown,
            ops: ops,
            parent: None,
            top_vdev: None,
            children: Vec::new(),
            create_txg: create_txg,

            top: vdev_top,
            leaf: None,
        }
    }

    pub fn load(normal_class: &Rc<MetaslabClass>,
                nv: &NvList,
                id: u64,
                parent: Option<TreeIndex>,
                vdev_tree: &Tree,
                alloc_type: AllocType)
                -> zfs::Result<Self> {
        let vdev_type = try!(nv.get::<&String>("type").ok_or(zfs::Error::Invalid)).clone();

        let ops = try!(load_ops(vdev_type.as_ref(), nv));

        if alloc_type == AllocType::Load {
            // Verify the provided id matches the id written in the MOS
            let label_id: u64 = try!(nv.get("id").ok_or(zfs::Error::Invalid));
            if label_id != id {
                return Err(zfs::Error::Invalid);
            }
        }

        // If this is some sort of load, then we read the guid from the nvpairs. Otherwise,
        // Vdev::new will generate one for us
        let guid = match alloc_type {
            AllocType::Load | AllocType::Spare | AllocType::L2Cache | AllocType::RootPool => {
                Some(try!(nv.get("guid").ok_or(zfs::Error::Invalid)))
            }
            _ => None,
        };

        let create_txg = try!(nv.get("create_txg").ok_or(zfs::Error::Invalid));
        let ashift = try!(nv.get("ashift").ok_or(zfs::Error::Invalid));

        let mut vdev_top = None;

        // If we're a top-level vdev, try to load the allocation parameters,
        // create the metaslab group, and create the vdev::Top
        if let Some(parent) = parent {
            if parent.get(vdev_tree).parent.is_none() {
                let mut ms_array = 0;
                let mut ms_shift = 0;
                if alloc_type == AllocType::Load || alloc_type == AllocType::Split {
                    ms_array = try!(nv.get("metaslab_array").ok_or(zfs::Error::Invalid));
                    ms_shift = try!(nv.get("metaslab_shift").ok_or(zfs::Error::Invalid));
                    // let asize = try!(nv.get("asize").ok_or(zfs::Error::Invalid));
                    // let removing = try!(nv.get("removing").ok_or(zfs::Error::Invalid));
                }

                if alloc_type != AllocType::Attach {
                    assert!(alloc_type == AllocType::Load || alloc_type == AllocType::Add ||
                            alloc_type == AllocType::Split ||
                            alloc_type == AllocType::RootPool);
                    let ms_group = MetaslabGroup::create(normal_class.clone());

                    vdev_top = Some(Top::new(ms_array, ms_shift, ms_group));
                }
            }
        }

        let mut vdev = Self::new(id, guid, ashift, ops, create_txg, vdev_top);
        vdev.parent = parent;

        Ok(vdev)
    }

    fn open(&mut self) -> zfs::Result<()> {
        Ok(())
    }

    fn metaslab_init(&mut self, mos: &mut ObjectSet, txg: u64) -> zfs::Result<()> {
        // We assume this is a top-level vdev
        let ref mut top = try!(self.top.as_mut().ok_or(zfs::Error::Invalid));

        let old_count = top.metaslabs.len();
        let new_count = (self.asize >> top.ms_shift) as usize;

        // assert!(txg == 0 || spa_config_held(spa, SCL_ALLOC, RW_WRITER));

        // Return if vdev isn't being allocated from yet
        if top.ms_shift == 0 {
            return Ok(());
        }
        assert!(!top.is_hole); // Must not be a hole

        // Compute the raidz-deflation ratio.  Note, we hard-code
        // in 128k (1 << 17) because it is the "typical" blocksize.
        // Even though SPA_MAXBLOCKSIZE changed, this algorithm can not change,
        // otherwise it would inconsistently account for existing bp's.
        // vd->vdev_deflate_ratio = (1 << 17) / (vdev_psize_to_asize(vd, 1 << 17) >> SPA_MINBLOCKSHIFT);

        assert!(old_count <= new_count);

        for m in old_count..new_count {
            let object: u64 = 0;

            if txg == 0 {
                // try!(dmu_read(mos, top.ms_array, m * mem::size_of::<u64>(),
                // mem::size_of::<u64>(), &object, DMU_READ_PREFETCH));
            }

            // let metaslab = try!(Metaslab::init(mos, self, m as u64, object, txg));
            // top.metaslabs.push(metaslab);
        }

        // if (txg == 0)
        //    spa_config_enter(spa, SCL_ALLOC, FTAG, RW_WRITER);

        // If the vdev is being removed we don't activate
        // the metaslabs since we want to ensure that no new
        // allocations are performed on this device.
        if old_count == 0 && !top.removing {
            // metaslab_group_activate(vd.mg);
        }

        // if (txg == 0)
        //    spa_config_exit(spa, SCL_ALLOC, FTAG);

        Ok(())
    }

    // Get the minimum allocatable size. We define the allocatable size as
    // the vdev's asize rounded to the nearest metaslab. This allows us to
    // replace or attach devices which don't have the same physical size but
    // can still satisfy the same number of allocations.
    // fn get_min_asize(&self, parent: Option<&Vdev>) -> u64 {
    // vdev_t *pvd = vd->vdev_parent;
    //
    // If our parent is NULL (inactive spare or cache) or is the root,
    // just return our own asize.
    // if self.parent.is_none() {
    // return self.asize;
    // }
    //
    // The top-level vdev just returns the allocatable size rounded
    // to the nearest metaslab.
    // if let Some(ref top) = self.top {
    // return util::p2_align(self.asize, 1u64 << top.ms_shift);
    // }
    //
    // The allocatable space for a raidz vdev is N * sizeof(smallest child),
    // so each child must provide at least 1/Nth of its asize.
    // if pvd->vdev_ops == &vdev_raidz_ops {
    //    return pvd->vdev_min_asize / pvd->vdev_children;
    // }
    //
    // pvd->vdev_min_asize
    // }


    // pub fn dirty(&mut self, flags: u64, void *arg, txg: u64) {
    // We assume this is a top-level vdev
    // let ref top = self.top.unwrap();
    //
    // assert!(self == self.top_vdev);
    // assert!(!self.is_hole);
    // assert!(util::is_p2(flags));
    // assert!(spa_writeable(self.spa));
    //
    // if flags & DIRTY_METASLAB {
    // txg_list_add(&self.ms_list, arg, txg);
    // }
    //
    // if flags & DIRTY_DTL {
    // txg_list_add(&self.dtl_list, arg, txg);
    // }
    //
    // txg_list_add(&self.spa.vdev_txg_list, self, txg);
    // }

    pub fn uberblock_shift(&self) -> u64 {
        cmp::min(cmp::max(self.ashift, uberblock::UBERBLOCK_SHIFT),
                 MAX_UBERBLOCK_SHIFT)
    }

    pub fn uberblock_count(&self) -> u64 {
        UBERBLOCK_RING >> self.uberblock_shift()
    }

    // pub fn uberblock_offset(&self, n) -> u64 {
    // offsetof(vdev_label_t, vl_uberblock[n << self.uberblock_shift()])
    // }

    pub fn uberblock_size(&self) -> u64 {
        1 << self.uberblock_shift()
    }
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Copy, Clone, PartialEq)]
pub struct TreeIndex(usize);

impl TreeIndex {
    pub fn get<'a>(&self, tree: &'a Tree) -> &'a Vdev {
        tree.nodes[self.0].as_ref().unwrap()
    }

    pub fn get_mut<'a>(&self, tree: &'a mut Tree) -> &'a mut Vdev {
        tree.nodes[self.0].as_mut().unwrap()
    }
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Tree {
    nodes: Vec<Option<Vdev>>,
    free: Vec<usize>,
}

impl Tree {
    pub fn new() -> Self {
        Tree {
            nodes: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn add(&mut self, vdev: Vdev) -> TreeIndex {
        let parent = vdev.parent;
        let guid = vdev.guid;

        // Add the vdev node
        let index = TreeIndex(match self.free.pop() {
            Some(free_index) => {
                self.nodes[free_index] = Some(vdev);
                free_index
            }
            None => {
                self.nodes.push(Some(vdev));
                self.nodes.len() - 1
            }
        });

        index.get_mut(self).top_vdev = parent.map(|parent| {
            parent.get(self).top_vdev.unwrap_or(index)
        });

        if let Some(parent) = parent {
            parent.get_mut(self).guid_sum += guid;
            parent.get_mut(self).children.push(index);
        }

        index
    }

    pub fn parse(&mut self,
                 normal_class: &Rc<MetaslabClass>,
                 nv: &NvList,
                 parent: Option<TreeIndex>,
                 alloc_type: AllocType)
                 -> zfs::Result<TreeIndex> {
        let vdev = try!(Vdev::load(normal_class, nv, 0, parent, self, alloc_type));
        let index = self.add(vdev);

        // Done parsing if this is a leaf
        if index.get(self).ops.is_leaf() {
            return Ok(index);
        }

        // Get the vdev's children
        let children: &Vec<NvList> = try!(nv.get("children").ok_or(zfs::Error::Invalid));

        for child in children {
            self.parse(normal_class, child, Some(index), alloc_type);
        }

        Ok(index)
    }

    pub fn load(&mut self, mos: &mut ObjectSet, root: TreeIndex) {
        // We use an iterative solution because of borrowing issues
        let mut queue = vec![root];

        while let Some(index) = queue.pop() {
            let vdev = index.get_mut(self);

            // Recursively load all children
            for child in &vdev.children {
                queue.push(*child);
            }

            // Load metaslabs for top-level vdevs
            // if let Some(ref top) = vdev.top {
            if vdev.top.is_some() {
                // if !top.is_hole {
                if vdev.ashift == 0 || vdev.asize == 0 || vdev.metaslab_init(mos, 0).is_err() {
                    // TODO: Set vdev state to error
                }
                // }
            }

            // TODO: Load DTL for leaf vdevs
        }
    }
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

const DIRTY_METASLAB: u64 = 0x01;
const DIRTY_DTL: u64 = 0x02;

const RAIDZ_MAXPARITY: usize = 3;

const PAD_SIZE: u64 = 8 << 10;
// 2 padding areas (vl_pad1 and vl_pad2) to skip
const SKIP_SIZE: u64 = PAD_SIZE * 2;
const PHYS_SIZE: u64 = 112 << 10;
const UBERBLOCK_RING: u64 = 128 << 10;

// The largest uberblock we support is 8k.
const MAX_UBERBLOCK_SHIFT: u64 = 13;
