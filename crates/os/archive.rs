use extract::Extract;

pub struct Archive<'a> {
    pub version: &'a [u8],
    pub buckets: u64,
    pub table: NodeTable<'a>,
    pub offset: usize,
    pub data: &'a [u8],
}

impl<'a> Archive<'a> {
    pub fn from_bytes(b: &[u8]) -> Self {
        let buckets = (((((((((((((b[8] << 8) | b[9]) << 8) | b[10]) << 8) | b[11]) << 8) | b[12]) << 8) | b[13]) << 8) | b[14]) << 8) | b[15];
        Archive {
            version: &b[0..8],
            table: Table::from_bytes(&b[256..256 + buckets]),
            offset: 256 + buckets,
            data: &b[256 + buckets..]
        }
    }

    pub fn extract(&self) -> Extract {
        OA

    }
}
