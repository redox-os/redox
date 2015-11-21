use super::avl;
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
    root_vdev: vdev::Vdev,
}

impl Spa {
    pub fn create(name: String, config: NvList) -> Self {
        let root_vdev = vdev::Vdev::new();
        Self::new(name, root_vdev)
    }

    pub fn open(&mut self) {
        let load_state = zfs::SpaLoadState::Open;
        if self.state == zfs::PoolState::Uninitialized {
            // First time opening

            self.activate();

            self.load(load_state);
        }
    }

    fn new(name: String, root_vdev: vdev::Vdev) -> Self {
        Spa {
            name: name,
            state: zfs::PoolState::Uninitialized,
            load_state: zfs::SpaLoadState::None,
            root_vdev: root_vdev,
        }
    }

    fn load(&mut self, load_state: zfs::SpaLoadState) {
        self.load_impl(load_state);
        self.load_state = zfs::SpaLoadState::Done;
    }

    /// mosconfig: Whether `config` came from on-disk MOS and so is trusted, or was user-made and so
    /// is untrusted.
    fn load_impl(&mut self, pool_guid: u64, config: &NvList, load_state: zfs::SpaLoadState,
                 import_type: ImportType, mos_config: bool) -> zfs::Result<()> {
        self.load_state = load_state;

        // Parse the vdev tree config
        let nvroot = config.find("vdev_tree").ok_or(zfs::Error::NoEntity);
        let vdev_alloc_type =
            match import_type {
                ImportType::Existing => vdev::AllocType::Load,
                ImportType::Assemble => vdev::AllocType::Split,
            };
        let root_vdev = try!(self.parse_vdev_tree(nvroot, vdev_alloc_type));

        Ok(())
    }

    fn parse_vdev_tree(&mut self, nv: &NvList, alloc_type: vdev::AllocType) -> zfs::Result<vdev::Vdev> {
        let vdev = vdev::Vdev::new(nv, alloc_type);

        // TODO: return here if the vdev is a leaf node

        // Get the vdev's children
        let children = config.find("children").ok_or(zfs::Error::NoEntity);
        let children =
            match children {
                NvList::NvListArray(children) => children,
                _ => { return Err(zfs::Error::Invalid); },
            };

        vdev
    }
    
    fn activate(&mut self) {
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SpaNamespace {
    avl: avl::Tree<Spa, &str>, // AVL tree of Spa sorted by name
}

impl SpaNamespace {
    pub fn new() -> Self {
        SpaNamespace {
            avl: avl::Tree::new(Box::new(|x| x.name.as_str())),
        }
    }

    pub fn add(&mut self, spa: Spa) {
        self.avl.insert(spa);
    }

    pub fn find<'a>(&'a self, name: &str) -> Option<&'a Spa> {
        self.avl.find(name)
    }

    pub fn find_mut<'a>(&'a mut self, name: &str) -> Option<&'a mut Spa> {
        self.avl.find_mut(name)
    }
}
