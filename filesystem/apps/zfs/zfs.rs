//To use this, please install zfs-fuse

use redox::*;

pub struct ZFS {
    disk: File,
}

impl ZFS {
    pub fn new(disk: File) -> ZFS {
        ZFS {
            disk: disk,
        }
    }
}

pub fn main() {
    let zfs = ZFS::new(File::open("zfs.img"));
}
