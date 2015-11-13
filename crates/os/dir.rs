use table::NodeTable;

pub struct Dir<'a> {
    name: &'a [u8],
    table: NodeTable<'a>,
}

impl<'a> Dir<'a> {
    pub fn from_bytes(b: &[u8]) -> Self {
        Dir {
            name: &b[..32],
            table: NodeTable {
                tab: &b[128..],
            },
        }
    }
}
