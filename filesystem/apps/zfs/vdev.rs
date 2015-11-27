use redox::{Box, String, ToString, Vec};

use super::from_bytes::FromBytes;
use super::metaslab::{Metaslab, MetaslabGroup};
use super::nvpair::{NvList, NvValue};
use super::vdev_file::VdevFile;
use super::zfs;
use super::zio;

#[repr(packed)]
pub struct VdevLabel {
    pub blank: [u8; 8 * 1024],
    pub boot_header: [u8; 8 * 1024],
    pub nv_pairs: [u8; 112 * 1024],
    pub uberblocks: [u8; 128 * 1024],
}

impl FromBytes for VdevLabel { }

////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct VdevOps {
    pub ops: Box<IVdevOps>,
    //io_start: fn(&zio::Zio),
    //io_done: fn(&zio::Zio),
    //state_change: fn(),
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
            Ok(VdevOps{
                ops: Box::new(try!(VdevFile::load(nv))),
                vdev_type: "disk".to_string(),
                is_leaf: true,
            })
        },
        _ => {
            Err(zfs::Error::Invalid)
        },
    }
}

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

/// States are ordered from least to most healthy.
/// Vdevs `CannotOpen` and worse are considered unusable.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Unknown,    // Uninitialized vdev
    Closed,     // Not currently open
    Offline,    // Not allowed to open
    Removed,    // Explicitly removed from the system
    CannotOpen, // Tried top open, but failed
    Faulted,    // External request to fault device
    Degraded,   // Replicated vdev with unhealthy kids
    Healthy,    // Presumed good
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Stuff that only top level vdevs have
pub struct Top {
    ms_array_object: u64,
    ms_group: MetaslabGroup,
    metaslabs: Vec<Metaslab>,
    is_hole: bool,
}

impl Top {
    pub fn new() -> Self {
        Top {
            ms_array_object: 0,
            ms_group: MetaslabGroup,
            metaslabs: vec![],
            is_hole: false, // TODO: zol checks vdev_ops for this, but idk what to do yet
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Leaf {
    whole_disk: u64,
}

impl Leaf {
    pub fn new() -> Self {
        Leaf {
            whole_disk: 0,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Note that a vdev can be a top-level, a leaf, both, or neither
pub struct Vdev {
    id: u64, // child number in vdev parent
    guid: u64, // unique ID for this vdev
    guid_sum: u64, // self guid + all child guids
    orig_guid: u64, // orig. guid prior to remove
    asize: u64, // allocatable device capacity
    min_asize: u64, // min acceptable asize
    max_asize: u64, // max acceptable asize
    ashift: u64, // block alignment shift
    state: State,
    prev_state: State,
    pub ops: VdevOps,
    parent: Option<TreeIndex>,
    children: Vec<TreeIndex>,
    create_txg: u64, // txg when top-level was added

    pub top: Option<Top>,
    pub leaf: Option<Leaf>,
}

impl Vdev {
    pub fn new(id: u64, guid: Option<u64>, ashift: u64, ops: VdevOps, create_txg: u64) -> Self {
        let guid =
            guid.unwrap_or_else(|| {
                // TODO: generate a guid
                0
            });
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
            children: Vec::new(),
            create_txg: create_txg,

            top: None,
            leaf: None,
        }
    }

    pub fn load(nv: &NvList, id: u64, parent: Option<TreeIndex>, alloc_type: AllocType) -> zfs::Result<Self> {
        let vdev_type = try!(nv.get::<&String>("type").ok_or(zfs::Error::Invalid)).clone();

        let ops = try!(load_ops(vdev_type.as_ref(), nv));

        if alloc_type == AllocType::Load {
            // Verify the provided id matches the id written in the MOS
            let label_id: u64 = try!(nv.get("id").ok_or(zfs::Error::Invalid));
            if label_id != id { return Err(zfs::Error::Invalid); }
        }

        // If this is some sort of load, then we read the guid from the nvpairs. Otherwise,
        // Vdev::new will generate one for us
        let guid =
            match alloc_type {
                AllocType::Load | AllocType::Spare | AllocType::L2Cache | AllocType::RootPool => {
                    Some(try!(nv.get("guid").ok_or(zfs::Error::Invalid)))
                },
                _ => { None },
            };

        let create_txg = try!(nv.get("create_txg").ok_or(zfs::Error::Invalid));
        let ashift = try!(nv.get("ashift").ok_or(zfs::Error::Invalid));

        let mut vdev = Self::new(id, guid, ashift, ops, create_txg);
        vdev.parent = parent;

        Ok(vdev)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
pub struct TreeIndex(usize);

impl TreeIndex {
    pub fn get<'a>(&self, tree: &'a Tree) -> &'a Vdev {
        tree.nodes[self.0].as_ref().unwrap()
    }

    pub fn get_mut<'a>(&self, tree: &'a mut Tree) -> &'a mut Vdev {
        tree.nodes[self.0].as_mut().unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

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

        // Add the vdev node
        let index =
            match self.free.pop() {
                Some(free_index) => {
                    self.nodes[free_index] = Some(vdev);
                    free_index
                },
                None => {
                    self.nodes.push(Some(vdev));
                    self.nodes.len()-1
                },
            };

        if let Some(parent) = parent {
            parent.get_mut(self).children.push(TreeIndex(index));
        }

        TreeIndex(index)
    }

    pub fn parse(&mut self, nv: &NvList, parent: Option<TreeIndex>,
             alloc_type: AllocType) -> zfs::Result<TreeIndex> {
        let vdev = try!(Vdev::load(nv, 0, parent, alloc_type));
        let index = self.add(vdev);

        // Done parsing if this is a leaf
        if index.get(self).ops.is_leaf() {
            return Ok(index);
        }

        // Get the vdev's children
        let children: &Vec<NvList> = try!(nv.get("children").ok_or(zfs::Error::Invalid));

        for child in children {
            self.parse(child, Some(index), alloc_type);
        }

        Ok(index)
    }
}
