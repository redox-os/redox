use std::cmp;
use std::rc::Rc;

use super::avl;
use super::dmu_objset::ObjectSet;
use super::dsl_pool;
use super::metaslab::{self, MetaslabClass};
use super::nvpair::{NvList, NvValue};
use super::taskq::Taskq;
use super::txg;
use super::uberblock::Uberblock;
use super::vdev;
use super::zfs;
use super::zio;

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
    zio_taskq: Vec<Vec<SpaTaskqs>>,
    // dsl_pool: DslPool,
    normal_class: Rc<MetaslabClass>, // normal data class
    log_class: Rc<MetaslabClass>, // intent log data class
    first_txg: u64,
    mos: ObjectSet,
    vdev_tree: vdev::Tree,
    root_vdev: vdev::TreeIndex,
    // ubsync: Uberblock, // Last synced uberblock
    // uberblock: Uberblock, // Current active uberblock
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

        // note that mos_config is true - we trust the user's config in this case
        let mut spa = try!(Self::load(name, config, load_state, ImportType::Existing, true));

        spa.activate();

        Ok(spa)
    }

    // pub fn open(&mut self) -> zfs::Result<()> {
    // let load_state = zfs::SpaLoadState::Open;
    // if self.state == zfs::PoolState::Uninitialized {
    // First time opening
    // self.activate();
    // try!(self.load(load_state, ImportType::Existing, false));
    // }
    //
    // Ok(())
    // }

    fn new(name: String, config: NvList, vdev_alloc_type: vdev::AllocType) -> zfs::Result<Self> {
        let metaslab_ops = Rc::new(metaslab::MetaslabOps { alloc: metaslab::ff_alloc });
        let normal_class = Rc::new(MetaslabClass::create(metaslab_ops.clone()));
        let log_class = Rc::new(MetaslabClass::create(metaslab_ops));

        // Parse vdev tree
        let mut vdev_tree = vdev::Tree::new();
        let root_vdev = {
            let nvroot: &NvList = try!(config.get("vdev_tree").ok_or(zfs::Error::Invalid));
            try!(vdev_tree.parse(&normal_class, nvroot, None, vdev_alloc_type))
        };

        Ok(Spa {
            name: name,
            config: config,
            state: zfs::PoolState::Uninitialized,
            load_state: zfs::SpaLoadState::None,
            zio_taskq: Vec::new(),
            // dsl_pool: blah,
            normal_class: normal_class,
            log_class: log_class,
            first_txg: 0,
            mos: ObjectSet,
            vdev_tree: vdev_tree,
            root_vdev: root_vdev,
            did: 0,
        })
    }

    fn load(name: String,
            config: NvList,
            load_state: zfs::SpaLoadState,
            import_type: ImportType,
            mos_config: bool)
            -> zfs::Result<Self> {
        let pool_guid = try!(config.get("pool_guid").ok_or(zfs::Error::Invalid));

        let mut spa = try!(Self::load_impl(name,
                                           pool_guid,
                                           config,
                                           load_state,
                                           import_type,
                                           mos_config));
        spa.load_state = zfs::SpaLoadState::None;

        Ok(spa)
    }

    /// mosconfig: Whether `config` came from on-disk MOS and so is trusted, or was user-made and so
    /// is untrusted.
    fn load_impl(name: String,
                 pool_guid: u64,
                 config: NvList,
                 load_state: zfs::SpaLoadState,
                 import_type: ImportType,
                 mos_config: bool)
                 -> zfs::Result<Self> {
        // Determine the vdev allocation type from import type
        let vdev_alloc_type = match import_type {
            ImportType::Existing => vdev::AllocType::Load,
            ImportType::Assemble => vdev::AllocType::Split,
        };

        let mut spa = try!(Self::new(name, config, vdev_alloc_type));
        spa.load_state = load_state;

        // Create "The Godfather" zio to hold all async IOs
        // spa.spa_async_zio_root = kmem_alloc(max_ncpus * sizeof (void *), KM_SLEEP);
        // for i in 0..max_ncpus {
        // spa.async_zio_root[i] =
        // Zio::root(spa, None, None, ZIO_FLAG_CANFAIL | ZIO_FLAG_SPECULATIVE | ZIO_FLAG_GODFATHER);
        // }


        // TODO: Try to open all vdevs, loading each label in the process.

        // TODO
        // Find the best uberblock.
        // vdev_uberblock_load(rvd, ub, &label);

        // If we weren't able to find a single valid uberblock, return failure.
        // if ub.txg == 0 {
        // return spa_vdev_err(rvd, VDEV_AUX_CORRUPT_DATA, ENXIO);
        // }


        // Initialize internal structures
        spa.state = zfs::PoolState::Active;
        // spa.ubsync = spa.uberblock;
        // spa.verify_min_txg =
        // if spa.extreme_rewind {
        // txg::TXG_INITIAL - 1
        // } else {
        // spa.last_synced_txg() - txg::DEFER_SIZE - 1;
        // };
        // spa.first_txg =
        // if spa.last_ubsync_txg { spa.last_ubsync_txg } else { spa.last_synced_txg() + 1 };
        // spa.claim_max_txg = spa.first_txg;
        // spa.prev_software_version = ub.software_version;

        // spa.dsl_pool = try!(dsl_pool::DslPool::init(&mut spa, spa.first_txg));
        // if error { return spa_vdev_err(rvd, VDEV_AUX_CORRUPT_DATA, EIO); }
        // spa.meta_objset = spa.dsl_pool.meta_objset;

        // Load stuff for the top-level and leaf vdevs
        spa.vdev_tree.load(&mut spa.mos, spa.root_vdev);

        Ok(spa)
    }

    fn activate(&mut self) {
        // assert!(self.state == zfs::PoolState::Uninitialized);

        self.state = zfs::PoolState::Active;

        // TODO: maybe start the spa thread

        self.create_zio_taskqs();

        self.did = 0;
    }

    // fn taskqs_init(&mut self, t: zio::Type, q: zio::TaskqType) {
    // const zio_taskq_info_t *ztip = &zio_taskqs[t][q];
    // zti_modes mode = ztip.mode;
    // let value = ztip.value;
    // let count = ztip.count;
    // let ref tqs = self.zio_taskq[t][q];
    // let flags = TASKQ_DYNAMIC;
    // let mut batch: bool = false;
    //
    // if mode == ZTI_MODE_NULL {
    // tqs.count = 0;
    // tqs.taskq = NULL;
    // return;
    // }
    //
    // assert!(count > 0);
    //
    // tqs.count = count;
    // tqs.taskq = kmem_alloc(count * sizeof (taskq_t *), KM_SLEEP);
    //
    // match mode {
    // ZTI_MODE_FIXED => {
    // assert!(value >= 1);
    // value = cmp::max(value, 1);
    // },
    // ZTI_MODE_BATCH => {
    // batch = true;
    // flags |= TASKQ_THREADS_CPU_PCT;
    // value = zio_taskq_batch_pct;
    // },
    // _ => {
    // panic!("unrecognized mode for %s_%s taskq (%u:%u) in spa_activate()",
    // zio_type_name[t], zio_taskq_types[q], mode, value);
    // },
    // }
    //
    // for i in 0..count {
    // taskq_t *tq;
    // char name[32];
    //
    // if (count > 1) {
    // snprintf(name, sizeof (name), "%s_%s_%u",
    // zio_type_name[t], zio_taskq_types[q], i);
    // } else {
    // snprintf(name, sizeof (name), "%s_%s",
    // zio_type_name[t], zio_taskq_types[q]);
    // }
    //
    // if zio_taskq_sysdc && spa->spa_proc != &p0 {
    // if batch {
    // flags |= TASKQ_DC_BATCH;
    // }
    //
    // tq = taskq_create_sysdc(name, value, 50, INT_MAX,
    // spa->spa_proc, zio_taskq_basedc, flags);
    // } else {
    // pri_t pri = maxclsyspri;
    // The write issue taskq can be extremely CPU
    // intensive.  Run it at slightly less important
    // priority than the other taskqs.  Under Linux this
    // means incrementing the priority value on platforms
    // like illumos it should be decremented.
    // if (t == ZIO_TYPE_WRITE && q == ZIO_TASKQ_ISSUE)
    // pri += 1;
    //
    // tq = taskq_create_proc(name, value, pri, 50,
    // INT_MAX, spa->spa_proc, flags);
    // }
    //
    // tqs->taskq[i] = tq;
    // }
    // }

    fn create_zio_taskqs(&mut self) {
        for t in 0..zio::NUM_TYPES {
            for q in 0..zio::NUM_TASKQ_TYPES {
                // self.taskqs_init(t, q);
            }
        }
    }

    fn last_synced_txg(&self) -> u64 {
        // TODO
        // self.ubsync.ub_txg
        0
    }

    fn first_txg(&self) -> u64 {
        self.first_txg
    }
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

struct ZioTaskqInfo {
    // mode: zti_modes_t,
    value: usize,
    count: usize,
}

struct SpaTaskqs {
    count: usize,
    taskq: Vec<Vec<Taskq>>,
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SpaNamespace {
    // TODO: Use &str instead of String as key type. Lifetimes are hard.
    avl: avl::Tree<Spa, String>, // AVL tree of Spa sorted by name
}

impl SpaNamespace {
    pub fn new() -> Self {
        SpaNamespace { avl: avl::Tree::new(Rc::new(|x| x.name.clone())) }
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
