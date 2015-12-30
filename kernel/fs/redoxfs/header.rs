use disk::ide::Extent;

/// The header of the fs
#[repr(packed)]
pub struct Header {
    pub signature: [u8; 8],
    pub version: u64,
    pub free_space: Extent,
    pub padding: [u8; 224],
    pub extents: [Extent; 16],
}

impl Header {
    pub fn valid(&self) -> bool {
        self.signature == "REDOXFS\0".as_bytes() && self.version == 1
    }
}
