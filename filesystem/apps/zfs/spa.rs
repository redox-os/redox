use redox::{Box, String, Vec};

use super::avl;
use super::nvpair::NvList;
use super::uberblock::Uberblock;
use super::vdev;
use super::zfs;

pub enum ImportType {
    Existing,
    Assemble,
}

// Storage pool allocator
pub struct Spa {
    name: String, // Pool name
    state: zfs::PoolState,
    load_state: zfs::SpaLoadState,
    vdev_tree: vdev::Tree,
    root_vdev: vdev::TreeIndex,
    //ubsync: Uberblock, // Last synced uberblock
    //uberblock: Uberblock, // Current active uberblock
    did: u64, // if procp != p0, did of t1
}

impl Spa {
    pub fn create(name: String, nvroot: &NvList) -> zfs::Result<Self> {
        let mut vdev_tree = vdev::Tree::new();
        let root_vdev = try!(vdev_tree.parse(nvroot, None, vdev::AllocType::Add));
        Ok(Self::new(name, vdev_tree, root_vdev))
    }

    pub fn open(&mut self) -> zfs::Result<()> {
        let load_state = zfs::SpaLoadState::Open;
        if self.state == zfs::PoolState::Uninitialized {
            // First time opening

            self.activate();

            try!(self.load(load_state, ImportType::Existing, false));

        }

        Ok(())
    }

    fn new(name: String, vdev_tree: vdev::Tree, root_vdev: vdev::TreeIndex) -> Self {
        Spa {
            name: name,
            state: zfs::PoolState::Uninitialized,
            load_state: zfs::SpaLoadState::None,
            vdev_tree: vdev::Tree::new(),
            root_vdev: root_vdev,
            did: 0,
        }
    }

    fn load(&mut self, load_state: zfs::SpaLoadState,
            import_type: ImportType, mos_config: bool) -> zfs::Result<()> {
        let ref config = NvList::new(0); // TODO: this should be replaced by self.config

        let pool_guid = try!(config.get("pool_guid").ok_or(zfs::Error::Invalid));

        self.load_impl(pool_guid, config, load_state, import_type, mos_config);
        self.load_state = zfs::SpaLoadState::None;

        Ok(())
    }

    /// mosconfig: Whether `config` came from on-disk MOS and so is trusted, or was user-made and so
    /// is untrusted.
    fn load_impl(&mut self, pool_guid: u64, config: &NvList, load_state: zfs::SpaLoadState,
                 import_type: ImportType, mos_config: bool) -> zfs::Result<()> {
        self.load_state = load_state;

        // Parse the vdev tree config
        let nvroot: &NvList = try!(config.get("vdev_tree").ok_or(zfs::Error::Invalid));
        let vdev_alloc_type =
            match import_type {
                ImportType::Existing => vdev::AllocType::Load,
                ImportType::Assemble => vdev::AllocType::Split,
            };
        //self.root_vdev = try!(self.parse_vdev_tree(nvroot, None, vdev_alloc_type));

        Ok(())
    }
    
    fn activate(&mut self) {
        assert!(self.state == zfs::PoolState::Uninitialized);

        self.state = zfs::PoolState::Active;

        //self.normal_class = MetaslabClass::create(self, zfs_metaslab_ops);
        //self.log_class = MetaslabClass::create(self, zfs_metaslab_ops);
        
        self.did = 0;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SpaNamespace {
    // TODO: Use &str instead of String as key type. Lifetimes are hard.
    avl: avl::Tree<Spa, String>, // AVL tree of Spa sorted by name
}

impl SpaNamespace {
    pub fn new() -> Self {
        SpaNamespace {
            avl: avl::Tree::new(Box::new(|x| x.name.clone())),
        }
    }

    pub fn add(&mut self, spa: Spa) {
        self.avl.insert(spa);
    }

    pub fn find(&self, name: String) -> Option<&Spa> {
        self.avl.find(name)
    }

    pub fn find_mut(&mut self, name: String) -> Option<&mut Spa> {
        self.avl.find_mut(name)
    }
}
