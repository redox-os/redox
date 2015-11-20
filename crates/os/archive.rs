use redox::prelude::v1::*;
use header::GlobalHeader;
use table::NodeTable;
use data::Data;

/// An Osmium archive
pub struct Archive<'a> {
    /// Header
    pub header: GlobalHeader<'a>,
    /// Root table
    pub root_table: NodeTable<'a>,
    /// Directory tables
    pub directories: &'a [u8],
    /// File segment
    pub files: &'a [u8],
}

impl<'a> Archive<'a> {
    /// Create an archive from bytes
    pub fn from_bytes(b: &'a [u8]) -> Self {
        let header = GlobalHeader::from_bytes(&b[..256]);
        let root_table = NodeTable::from_bytes(&b[256..256 + header.root_buckets as usize * 16]);
        let directories =
            &b[256 +
               header.root_buckets as usize *
               16..256 + (header.root_buckets * 16 + header.dir_size * 16) as usize];
        let files = &b[256 + header.root_buckets as usize * 16 + header.dir_size as usize * 16..];

        Archive {
            header: header,
            root_table: root_table,
            directories: directories,
            files: files,
        }
    }

    /// Get a given file from a table
    pub fn get(&self, query: &str, table: NodeTable<'a>) -> Option<Data> {
        let mut probe = 0;

        loop {
            match table.get(query, &mut probe) {
                Some(ref ptr) => {
                    let dat = ptr.deref(self);
                    if dat.name() == query {
                        return Some(dat);
                    }
                }
                None => return None,
            }
        }
    }
}
