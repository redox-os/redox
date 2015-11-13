/// The global archive header
pub struct GlobalHeader<'a> {
    pub version: &'a [u8],
    pub root_buckets: u64,
    pub dir_size: u64,
}

impl<'a> GlobalHeader<'a> {
    /// Create new from bytes
    pub fn from_bytes(b: &'a [u8]) -> Self {
        GlobalHeader {
            version: &b[..8],
            root_buckets: b[8..16].iter().fold(0, |x, &i| x << 8 | i as u64),
            dir_size: b[16..24].iter().fold(0, |x, &i| x << 8 | i as u64),
        }
    }
}
