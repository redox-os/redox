pub struct Archive<'a> {
    pub header: GlobalHeader<'a>,
    pub root_table: NodeTable<'a>,
    pub directories: &'a [u8],
    pub files: &'a [u8],
}

impl<'a> Archive<'a> {
    pub fn from_bytes(b: &[u8]) -> Self {
        let header = GlobalHeader::from_bytes(&b[..257]);
        let root_table = NodeTable::from_bytes(&b[256..256 + header.root_buckets * 16]);
        let directories = &b[256 + header.root_buckets * 16..256 + header.root_buckets * 16 + header.dir_size * 16];
        let files = &b[256 + header.root_buckets * 16 + header.dir_size * 16..];

        Archive {
            header: header,
            root_table: root_table,
            diretories: directories,
            files: files,
        }
    }
}


