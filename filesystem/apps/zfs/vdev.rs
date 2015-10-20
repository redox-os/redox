use super::from_bytes::FromBytes;
use super::Uberblock;

#[repr(packed)]
pub struct VdevLabel {
    pub blank: [u8; 8 * 1024],
    pub boot_header: [u8; 8 * 1024],
    pub nv_pairs: [u8; 112 * 1024],
    pub uberblocks: [Uberblock; 128],
}

impl FromBytes for VdevLabel { }
