use redox::Vec;

use super::from_bytes::FromBytes;
use super::metaslab::{Metaslab, MetaslabGroup};
use super::nvpair::{NvList, NvValue};
use super::zfs;

#[repr(packed)]
pub struct VdevLabel {
    pub blank: [u8; 8 * 1024],
    pub boot_header: [u8; 8 * 1024],
    pub nv_pairs: [u8; 112 * 1024],
    pub uberblocks: [u8; 128 * 1024],
}

impl FromBytes for VdevLabel { }

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
    //ops: VdevOps,

    // Top level only
    ms_array_object: u64,
    ms_group: MetaslabGroup,
    metaslabs: Vec<Metaslab>,
    is_hole: bool,

    // Leaf only
}

impl Vdev {
    pub fn new(id: u64, guid: u64) -> Self {
        Vdev {
            id: id,
            guid: guid,
            guid_sum: guid, // No children yet, so guid_sum is just my guid
            orig_guid: 0,
            asize: 0,
            min_asize: 0,
            max_asize: 0,
            ashift: 0,
            state: State::Closed,
            prev_state: State::Unknown,

            // Top level only
            ms_array_object: 0,
            ms_group: MetaslabGroup,
            metaslabs: vec![],
            is_hole: false, // TODO: zol checks vdev_ops for this, but idk what to do yet
        }
    }

    pub fn load(nv: &NvList, id: u64, alloc_type: AllocType) -> zfs::Result<Self> {
        if alloc_type == AllocType::Load {
            // Verify the provided id matches the id written in the MOS
            let label_id = try!(nv.find("id").ok_or(zfs::Error::Invalid));
            match *label_id {
                NvValue::Uint64(label_id) => {
                    if label_id != id { return Err(zfs::Error::Invalid); }
                },
                _ => { },
            }
        }

        /*let guid =
            match alloc_type {
                AllocType::Load | AllocType::Spare | AllocType::L2Cache | AllocType::RootPool => {
                    Some(try!(nv.find("guid").ok_or(zfs::Error::Invalid)))
                },
                _ => { None },
            };*/

        //Ok(Self::new(id, guid))
        Ok(Self::new(id, 0))
    }
}
