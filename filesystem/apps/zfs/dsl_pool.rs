use super::spa;
use super::zfs;

pub struct DslPool {
    // Immutable
    root_dir_obj: u64,
}

impl DslPool {
    pub fn init(spa: &mut spa::Spa, txg: u64) -> zfs::Result<Self> {
        Self::open_impl(spa, txg)
    }

    fn open_impl(spa: &mut spa::Spa, txg: u64) -> zfs::Result<Self> {
        Ok(DslPool { root_dir_obj: 0 })
    }
}
