use super::nvpair::NvList;
use super::{vdev, zfs};

pub struct VdevFile {
    path: String,
}

impl VdevFile {
    pub fn load(nv: &NvList) -> zfs::Result<Self> {
        Ok(VdevFile { path: try!(nv.get::<&String>("path").ok_or(zfs::Error::Invalid)).clone() })
    }

    // pub fn io_start(zio: &zio::Zio);

    // pub fn io_done(zio: &zio::Zio);

    // pub fn state_change();
}

impl vdev::IVdevOps for VdevFile {
    fn open(&mut self, vdev: &mut vdev::Vdev) -> zfs::Result<(u64, u64, u64)> {
        Ok((0, 0, 0))
    }

    fn close(&mut self, vdev: &mut vdev::Vdev) {}

    fn asize(&mut self, vdev: &mut vdev::Vdev, psize: u64) -> u64 {
        0
    }

    fn hold(&mut self, vdev: &mut vdev::Vdev) {}

    fn release(&mut self, vdev: &mut vdev::Vdev) {}
}
