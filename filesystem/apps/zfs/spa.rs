use super::avl;
use super::nvpair::{NvList, NvValue};
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
    config: NvList,
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
        let mut config = NvList::new(0);
        config.add("name".to_string(), NvValue::String(name.clone()));
        Self::new(name, config, vdev::AllocType::Add)
    }

    pub fn import(name: String, config: NvList) -> zfs::Result<Self> {
        let load_state = zfs::SpaLoadState::Import;

        // mos_config is true - we trust the user's config in this case
        let mut spa = try!(Self::load(name, config, load_state, ImportType::Existing, true));

        spa.activate();

        Ok(spa)
    }

    /*pub fn open(&mut self) -> zfs::Result<()> {
        let load_state = zfs::SpaLoadState::Open;
        if self.state == zfs::PoolState::Uninitialized {
            // First time opening

            self.activate();

            try!(self.load(load_state, ImportType::Existing, false));
        }

        Ok(())
    }*/

    fn new(name: String, config: NvList, vdev_alloc_type: vdev::AllocType) -> zfs::Result<Self> {
        // Parse vdev tree
        let mut vdev_tree = vdev::Tree::new();
        let root_vdev = {
            let nvroot: &NvList = try!(config.get("vdev_tree").ok_or(zfs::Error::Invalid));
            try!(vdev_tree.parse(nvroot, None, vdev_alloc_type))
        };

        Ok(Spa {
            name: name,
            config: config,
            state: zfs::PoolState::Uninitialized,
            load_state: zfs::SpaLoadState::None,
            vdev_tree: vdev_tree,
            root_vdev: root_vdev,
            did: 0,
        })
    }

    fn load(name: String, config: NvList, load_state: zfs::SpaLoadState,
            import_type: ImportType, mos_config: bool) -> zfs::Result<Self> {
        let pool_guid = try!(config.get("pool_guid").ok_or(zfs::Error::Invalid));

        let mut spa = try!(Self::load_impl(name, pool_guid, config, load_state,
                                           import_type, mos_config));
        spa.load_state = zfs::SpaLoadState::None;

        Ok(spa)
    }

    /// mosconfig: Whether `config` came from on-disk MOS and so is trusted, or was user-made and so
    /// is untrusted.
    fn load_impl(name: String, pool_guid: u64, config: NvList,
                 load_state: zfs::SpaLoadState, import_type: ImportType,
                 mos_config: bool) -> zfs::Result<Self> {
        // Determine the vdev allocation type from import type
        let vdev_alloc_type =
            match import_type {
                ImportType::Existing => vdev::AllocType::Load,
                ImportType::Assemble => vdev::AllocType::Split,
            };

        let mut spa = try!(Self::new(name, config, vdev_alloc_type));
        spa.load_state = load_state;

        Ok(spa)
    }
    
    fn activate(&mut self) {
        //assert!(self.state == zfs::PoolState::Uninitialized);

        self.state = zfs::PoolState::Active;

        //self.normal_class = MetaslabClass::create(self, zfs_metaslab_ops);
        //self.log_class = MetaslabClass::create(self, zfs_metaslab_ops);

        // TODO: Start the spa thread
        
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
