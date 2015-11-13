use redox::prelude::v1::*;
use archive::Archive;

/// A table of nodes, i.e. directories and files
pub struct NodeTable<'a> {
    pub table: &'a [u8],
}

impl NodeTable {
    /// Create a new node table from a slice of bytes
    pub fn new(&self, b: &[u8]) -> Self {
        NodeTable {
            table: b,
        }
    }

    /// Get the n'th entry of the table
    pub fn get_entry(&self, n: usize) -> Option<DataPtr> {
        DataPtr::from_bytes(n, &self.table[n * 16..n * 16 + 16])
    }

    /// Get the entry given an key and a probe (in this case the key should is the node name)
    pub fn get(&self, key: &[u8], probe: &mut usize) -> Option<DataPtr> {
        let hasher = Djb2::new();
        key.hash(hasher);

        let res = self.get_entry(
            hasher.finish() + *probe
        );

        *probe = match *probe + 1 {
            n if n < self.table.len() => n,
            n if n == self.table.len() => 0,
            // Just in case:
            n => n % self.table.len(),
        };

        res
    }

}
