use super::avl;
use super::vdev;
use super::zfs;

// Storage pool allocator
pub struct Spa {
    name: String, // Pool name
    state: zfs::PoolState,
    root_vdev: Option<vdev::Vdev>,
}

impl Spa {
    pub fn new(name: String) -> Self {
        Spa {
            name: name,
            state: zfs::PoolState::Uninitialized,
            root_vdev: None,
        }
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

    pub fn add(&mut self, name: String, config: NvList) {
        let spa = Spa::new(name);
        self.avl.insert(spa);
    }

    pub fn find<'a>(&'a self, name: &str) -> Option<&'a Spa> {
        self.avl.find(name)
    }

    pub fn find_mut<'a>(&'a mut self, name: &str) -> Option<&'a mut Spa> {
        self.avl.find_mut(name)
    }
}
