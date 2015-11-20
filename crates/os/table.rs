use redox::prelude::v1::*;
use redox::hash::{Hash, Hasher};
use ptr::DataPtr;
use djb2::Djb2;

/// A table of nodes, i.e. directories and files
pub struct NodeTable<'a> {
    pub table: &'a [u8],
}

impl<'a> NodeTable<'a> {
    /// Create a new node table from a slice of bytes
    pub fn from_bytes(b: &'a [u8]) -> Self {
        NodeTable { table: b }
    }

    /// Get the n'th entry of the table
    pub fn get_entry(&self, n: usize) -> Option<DataPtr> {
        DataPtr::from_bytes(&self.table[n * 16..n * 16 + 16])
    }

    /// Get the entry given an key and a probe (in this case the key should is the node name)
    pub fn get(&self, key: &str, probe: &mut u64) -> Option<DataPtr> {
        let mut hasher = Djb2::new();
        key.hash(&mut hasher);

        let res = self.get_entry((hasher.finish() + *probe) as usize);

        let len = self.table.len() as u64;

        *probe = match *probe + 1 {
            n if n < len => n,
            n if n == len => 0,
            // Just in case:
            n => n % len,
        };

        res
    }

}
